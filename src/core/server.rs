use crate::core::cert::CertPair;
use crate::core::receive_events::{push_incoming_event, IncomingFileMeta, IncomingTransferEvent};
use base64::Engine as _;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::http::header::CONTENT_TYPE;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use localsend::http::dto::{ErrorResponse, NonceRequest, NonceResponse, PrepareUploadResponseDto};
use localsend::http::state::ClientInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::{oneshot, Mutex};
use tokio_rustls::TlsAcceptor;

#[derive(Clone)]
struct ServerState {
    info: Arc<Mutex<ClientInfo>>,
    sessions: Arc<Mutex<HashMap<String, IncomingSession>>>,
}

#[derive(Clone, Debug)]
struct IncomingSessionFile {
    file_name: String,
    file_type: String,
    token: String,
    received: bool,
}

#[derive(Clone, Debug)]
struct IncomingSession {
    sender_ip: IpAddr,
    sender_alias: String,
    sender_device_model: Option<String>,
    sender_fingerprint: String,
    files: HashMap<String, IncomingSessionFile>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WirePeerInfo {
    alias: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    device_model: Option<String>,
    #[serde(default)]
    device_type: Option<serde_json::Value>,
    #[serde(default)]
    fingerprint: Option<String>,
    #[serde(default)]
    token: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireFileDto {
    id: String,
    file_name: String,
    size: u64,
    file_type: String,
    #[serde(default)]
    preview: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WirePrepareUploadRequest {
    info: WirePeerInfo,
    files: HashMap<String, WireFileDto>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireRegisterRequest {
    alias: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    device_model: Option<String>,
    #[serde(default)]
    device_type: Option<String>,
    #[serde(default)]
    fingerprint: Option<String>,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    port: Option<u16>,
    #[serde(default)]
    protocol: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompatInfoResponse {
    alias: String,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_type: Option<String>,
    fingerprint: String,
    download: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompatRegisterResponse {
    alias: String,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_type: Option<String>,
    fingerprint: String,
    token: String,
    download: bool,
    has_web_interface: bool,
}

/// Server manager owned by NearSend.
pub struct ServerManager {
    port: u16,
    stop_tx: Option<oneshot::Sender<()>>,
}

impl ServerManager {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            stop_tx: None,
        }
    }

    pub fn start(
        &mut self,
        client_info: ClientInfo,
        use_https: bool,
        cert: Option<CertPair>,
        handle: &Handle,
    ) -> anyhow::Result<()> {
        if self.is_running() {
            log::warn!("Server already running on port {}", self.port);
            return Ok(());
        }

        let tls_acceptor = if use_https {
            let cert = cert.ok_or_else(|| anyhow::anyhow!("HTTPS requires certificate"))?;
            Some(build_tls_acceptor(&cert)?)
        } else {
            None
        };

        let (stop_tx, stop_rx) = oneshot::channel();
        self.stop_tx = Some(stop_tx);

        let port = self.port;
        handle.spawn(async move {
            if let Err(e) = start_with_port(port, client_info, tls_acceptor, stop_rx).await {
                log::error!("Server error: {}", e);
            }
        });

        log::info!(
            "Starting NearSend server on port {} (https={})",
            self.port,
            use_https
        );
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
            log::info!("Stopping NearSend server");
        }
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn is_running(&self) -> bool {
        self.stop_tx.is_some()
    }
}

fn build_tls_acceptor(cert: &CertPair) -> anyhow::Result<TlsAcceptor> {
    use rustls::pki_types::pem::PemObject;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};

    let _ = rustls::crypto::ring::default_provider().install_default();
    let cert_chain = vec![CertificateDer::from_pem_slice(cert.cert_pem.as_bytes())?];
    let private_key = PrivateKeyDer::from_pem_slice(cert.private_key_pem.as_bytes())?;
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_key)?;
    Ok(TlsAcceptor::from(Arc::new(config)))
}

async fn start_with_port(
    port: u16,
    info: ClientInfo,
    tls_acceptor: Option<TlsAcceptor>,
    mut stop_rx: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    let listener_v4 = tokio::net::TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await?;
    let listener_v6 = match tokio::net::TcpListener::bind((Ipv6Addr::UNSPECIFIED, port)).await {
        Ok(listener) => Some(listener),
        Err(err) => {
            log::warn!("ipv6 listener disabled on port {}: {}", port, err);
            None
        }
    };

    let state = ServerState {
        info: Arc::new(Mutex::new(info)),
        sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    loop {
        tokio::select! {
            _ = &mut stop_rx => {
                break;
            }
            accepted = listener_v4.accept() => {
                let (stream, remote_addr) = accepted?;
                let state = state.clone();
                let tls_acceptor = tls_acceptor.clone();
                tokio::spawn(async move {
                    let builder = Builder::new(TokioExecutor::new());
                    if let Some(acceptor) = tls_acceptor {
                        match acceptor.accept(stream).await {
                            Ok(tls_stream) => {
                                let svc = service_fn(move |req| handle_request(req, state.clone(), remote_addr.ip()));
                                let conn = builder.serve_connection(TokioIo::new(tls_stream), svc);
                                if let Err(err) = conn.await {
                                    log::warn!("serve tls connection failed: {}", err);
                                }
                            }
                            Err(err) => log::warn!("tls handshake failed: {}", err),
                        }
                    } else {
                        let svc = service_fn(move |req| handle_request(req, state.clone(), remote_addr.ip()));
                        let conn = builder.serve_connection(TokioIo::new(stream), svc);
                        if let Err(err) = conn.await {
                            log::warn!("serve connection failed: {}", err);
                        }
                    }
                });
            }
            accepted = async {
                match &listener_v6 {
                    Some(listener) => listener.accept().await,
                    None => std::future::pending::<std::io::Result<(tokio::net::TcpStream, std::net::SocketAddr)>>().await,
                }
            } => {
                let (stream, remote_addr) = accepted?;
                let state = state.clone();
                let tls_acceptor = tls_acceptor.clone();
                tokio::spawn(async move {
                    let builder = Builder::new(TokioExecutor::new());
                    if let Some(acceptor) = tls_acceptor {
                        match acceptor.accept(stream).await {
                            Ok(tls_stream) => {
                                let svc = service_fn(move |req| handle_request(req, state.clone(), remote_addr.ip()));
                                let conn = builder.serve_connection(TokioIo::new(tls_stream), svc);
                                if let Err(err) = conn.await {
                                    log::warn!("serve tls connection failed: {}", err);
                                }
                            }
                            Err(err) => log::warn!("tls handshake failed: {}", err),
                        }
                    } else {
                        let svc = service_fn(move |req| handle_request(req, state.clone(), remote_addr.ip()));
                        let conn = builder.serve_connection(TokioIo::new(stream), svc);
                        if let Err(err) = conn.await {
                            log::warn!("serve connection failed: {}", err);
                        }
                    }
                });
            }
        }
    }

    Ok(())
}

async fn handle_request(
    req: Request<Incoming>,
    state: ServerState,
    remote_ip: IpAddr,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let response = match handle_request_inner(req, state, remote_ip).await {
        Ok(resp) => resp,
        Err((status, message)) => json_response(status, &ErrorResponse { message }),
    };
    Ok(response)
}

async fn handle_request_inner(
    req: Request<Incoming>,
    state: ServerState,
    remote_ip: IpAddr,
) -> Result<Response<Full<Bytes>>, (StatusCode, String)> {
    let path = req.uri().path().to_string();
    let method = req.method().clone();

    match (method, path.as_str()) {
        (Method::GET, "/api/localsend/v1/info")
        | (Method::GET, "/api/localsend/v2/info")
        | (Method::GET, "/api/localsend/v3/info") => handle_info(req, state).await,
        (Method::POST, "/api/localsend/v1/register")
        | (Method::POST, "/api/localsend/v2/register")
        | (Method::POST, "/api/localsend/v3/register") => {
            handle_register(req, state, remote_ip).await
        }
        (Method::POST, "/api/localsend/v3/nonce") => handle_nonce(req).await,
        (Method::POST, "/api/localsend/v1/send-request")
        | (Method::POST, "/api/localsend/v1/prepare-upload") => {
            handle_prepare_upload(req, state, remote_ip, false).await
        }
        (Method::POST, "/api/localsend/v2/prepare-upload")
        | (Method::POST, "/api/localsend/v2/send-request") => {
            handle_prepare_upload(req, state, remote_ip, true).await
        }
        (Method::POST, "/api/localsend/v1/send") | (Method::POST, "/api/localsend/v1/upload") => {
            handle_upload(req, state, remote_ip, false).await
        }
        (Method::POST, "/api/localsend/v2/upload") | (Method::POST, "/api/localsend/v2/send") => {
            handle_upload(req, state, remote_ip, true).await
        }
        (Method::POST, "/api/localsend/v1/cancel") => handle_cancel(req, state, false).await,
        (Method::POST, "/api/localsend/v2/cancel") => handle_cancel(req, state, true).await,
        (Method::POST, "/api/localsend/v1/show") | (Method::POST, "/api/localsend/v2/show") => {
            Ok(json_response(StatusCode::OK, &serde_json::json!({})))
        }
        _ => Ok(json_response(
            StatusCode::NOT_FOUND,
            &ErrorResponse {
                message: "Not Found".to_string(),
            },
        )),
    }
}

async fn handle_info(
    req: Request<Incoming>,
    state: ServerState,
) -> Result<Response<Full<Bytes>>, (StatusCode, String)> {
    let info = state.info.lock().await.clone();
    let own_fingerprint = info.token.clone();
    let query = parse_query(req.uri().query().unwrap_or_default());
    let sender_fingerprint = query
        .get("fingerprint")
        .cloned()
        .or_else(|| query.get("token").cloned())
        .unwrap_or_default();

    if !sender_fingerprint.is_empty() && sender_fingerprint == own_fingerprint {
        return Ok(json_response(
            StatusCode::PRECONDITION_FAILED,
            &ErrorResponse {
                message: "Self-discovered".to_string(),
            },
        ));
    }

    Ok(json_response(
        StatusCode::OK,
        &CompatInfoResponse {
            alias: info.alias,
            version: info.version,
            device_model: info.device_model,
            device_type: map_device_type_to_wire(info.device_type.as_ref()),
            fingerprint: own_fingerprint,
            download: false,
        },
    ))
}

async fn handle_register(
    req: Request<Incoming>,
    state: ServerState,
    remote_ip: IpAddr,
) -> Result<Response<Full<Bytes>>, (StatusCode, String)> {
    match parse_json_body::<WireRegisterRequest>(req.into_body()).await {
        Ok(peer) => {
            let token = peer.fingerprint.or(peer.token).unwrap_or_default();
            let https = peer
                .protocol
                .as_deref()
                .map(|p| p.eq_ignore_ascii_case("https"))
                .unwrap_or(false);
            let port = peer.port.unwrap_or(53317);
            log::info!(
                "register discovered peer alias={} ip={} port={} https={} token_empty={}",
                peer.alias,
                remote_ip,
                port,
                https,
                token.is_empty()
            );
            crate::core::discovery::register_passive_device(
                crate::core::discovery::DiscoveredDevice {
                    info: ClientInfo {
                        alias: peer.alias,
                        version: peer.version.unwrap_or_else(|| "2.1".to_string()),
                        device_model: peer.device_model,
                        device_type: map_wire_device_type(peer.device_type.as_deref()),
                        token,
                    },
                    ip: remote_ip.to_string(),
                    port,
                    https,
                },
            );
        }
        Err((_, err)) => {
            log::debug!("register parse failed from {}: {}", remote_ip, err);
        }
    }

    let info = state.info.lock().await.clone();
    Ok(json_response(
        StatusCode::OK,
        &CompatRegisterResponse {
            alias: info.alias,
            version: info.version,
            device_model: info.device_model,
            device_type: map_device_type_to_wire(info.device_type.as_ref()),
            fingerprint: info.token.clone(),
            token: info.token,
            download: false,
            has_web_interface: false,
        },
    ))
}

async fn handle_nonce(
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, (StatusCode, String)> {
    let _payload: NonceRequest = parse_json_body(req.into_body()).await?;
    let nonce = base64::engine::general_purpose::STANDARD.encode(uuid::Uuid::new_v4().as_bytes());
    Ok(json_response(StatusCode::OK, &NonceResponse { nonce }))
}

async fn handle_prepare_upload(
    req: Request<Incoming>,
    state: ServerState,
    remote_ip: IpAddr,
    v2: bool,
) -> Result<Response<Full<Bytes>>, (StatusCode, String)> {
    // Keep behavior simple: only one active incoming session at a time.
    if !state.sessions.lock().await.is_empty() {
        return Err((
            StatusCode::CONFLICT,
            "Blocked by another session".to_string(),
        ));
    }

    let payload: WirePrepareUploadRequest = parse_json_body(req.into_body()).await?;
    if payload.files.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Request must contain at least one file".to_string(),
        ));
    }

    // Register sender as a discovered device as soon as transfer request arrives.
    // This must happen regardless of whether user accepts/rejects/cancels the transfer.
    {
        let token = payload
            .info
            .fingerprint
            .clone()
            .or(payload.info.token.clone())
            .unwrap_or_default();
        let peer = crate::core::discovery::DiscoveredDevice {
            info: ClientInfo {
                alias: payload.info.alias.clone(),
                version: payload
                    .info
                    .version
                    .clone()
                    .unwrap_or_else(|| "1.0".to_string()),
                device_model: payload.info.device_model.clone(),
                device_type: map_wire_device_type(
                    payload.info.device_type.as_ref().and_then(|v| v.as_str()),
                ),
                token,
            },
            ip: remote_ip.to_string(),
            // prepare-upload payload does not include sender port; use LocalSend default.
            port: 53317,
            https: false,
        };
        log::info!(
            "prepare-upload discovered peer alias={} ip={} token_empty={}",
            peer.info.alias,
            peer.ip,
            peer.info.token.is_empty()
        );
        crate::core::discovery::register_passive_device(peer);
    }

    let sender_fingerprint = payload
        .info
        .fingerprint
        .clone()
        .or(payload.info.token.clone())
        .unwrap_or_default();
    let session_id = uuid::Uuid::new_v4().to_string();
    let sender_alias = payload.info.alias.clone();
    let sender_device_model = payload.info.device_model.clone();

    let mut metas = Vec::new();
    for (file_id, f) in &payload.files {
        metas.push(IncomingFileMeta {
            file_id: file_id.clone(),
            file_name: f.file_name.clone(),
            file_type: normalize_file_type(&f.file_type),
            size: f.size,
        });
    }
    push_incoming_event(IncomingTransferEvent::Prepared {
        session_id: session_id.clone(),
        sender_alias: sender_alias.clone(),
        sender_device_model: sender_device_model.clone(),
        sender_fingerprint: sender_fingerprint.clone(),
        files: metas,
    });

    let is_single_text_message = payload.files.len() == 1
        && payload
            .files
            .values()
            .next()
            .map(|f| is_text_type(&f.file_type) && f.preview.is_some())
            .unwrap_or(false);
    if is_single_text_message {
        if let Some((file_id, f)) = payload.files.iter().next() {
            push_incoming_event(IncomingTransferEvent::FileReceived {
                session_id: session_id.clone(),
                file_id: file_id.clone(),
                saved_path: None,
                text_content: f.preview.clone(),
            });
        }
        push_incoming_event(IncomingTransferEvent::Completed { session_id });
        return Ok(no_content());
    }

    let mut accepted_files = HashMap::new();
    let mut session_files = HashMap::new();
    for (file_id, f) in payload.files {
        let token = uuid::Uuid::new_v4().to_string();
        accepted_files.insert(file_id.clone(), token.clone());
        session_files.insert(
            file_id,
            IncomingSessionFile {
                file_name: f.file_name,
                file_type: normalize_file_type(&f.file_type),
                token,
                received: false,
            },
        );
    }

    state.sessions.lock().await.insert(
        session_id.clone(),
        IncomingSession {
            sender_ip: remote_ip,
            sender_alias,
            sender_device_model,
            sender_fingerprint,
            files: session_files,
        },
    );

    if v2 {
        Ok(json_response(
            StatusCode::OK,
            &PrepareUploadResponseDto {
                session_id,
                files: accepted_files,
            },
        ))
    } else {
        Ok(json_response(StatusCode::OK, &accepted_files))
    }
}

async fn handle_upload(
    req: Request<Incoming>,
    state: ServerState,
    remote_ip: IpAddr,
    v2: bool,
) -> Result<Response<Full<Bytes>>, (StatusCode, String)> {
    let query = parse_query(req.uri().query().unwrap_or_default());
    let file_id = query_first(&query, &["fileId", "fileID", "file_id", "id"])
        .cloned()
        .ok_or((StatusCode::BAD_REQUEST, "missing fileId".to_string()))?;
    let token = query_first(&query, &["token", "fileToken"])
        .cloned()
        .ok_or((StatusCode::BAD_REQUEST, "missing token".to_string()))?;
    let session_id_query = query_first(&query, &["sessionId", "session_id", "sid"]).cloned();
    if v2 && session_id_query.is_none() {
        return Err((StatusCode::BAD_REQUEST, "missing sessionId".to_string()));
    }

    let body = req
        .into_body()
        .collect()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("read body failed: {}", e)))?
        .to_bytes();

    let (session_id, saved_path, text_content, completed_now) = {
        let mut sessions = state.sessions.lock().await;
        let session_id = if let Some(session_id) = session_id_query.clone() {
            session_id
        } else {
            sessions
                .keys()
                .next()
                .cloned()
                .ok_or((StatusCode::CONFLICT, "No session".to_string()))?
        };

        let session = sessions
            .get_mut(&session_id)
            .ok_or((StatusCode::FORBIDDEN, "Invalid session id".to_string()))?;
        if session.sender_ip != remote_ip {
            return Err((StatusCode::FORBIDDEN, "Invalid IP address".to_string()));
        }
        let file = session
            .files
            .get_mut(&file_id)
            .ok_or((StatusCode::FORBIDDEN, "Invalid token".to_string()))?;
        if file.token != token {
            return Err((StatusCode::FORBIDDEN, "Invalid token".to_string()));
        }

        let text_content = if is_text_type(&file.file_type) {
            String::from_utf8(body.to_vec()).ok()
        } else {
            None
        };
        let save_dir = std::env::temp_dir()
            .join("near-send-received")
            .join(&session_id);
        let _ = tokio::fs::create_dir_all(&save_dir).await;
        let file_path = save_dir.join(sanitize_file_name(&file.file_name));
        if let Err(err) = tokio::fs::write(&file_path, body).await {
            log::warn!("save incoming file failed: {}", err);
        }

        file.received = true;
        let completed_now = session.files.values().all(|f| f.received);
        (
            session_id,
            Some(file_path.to_string_lossy().to_string()),
            text_content,
            completed_now,
        )
    };

    push_incoming_event(IncomingTransferEvent::FileReceived {
        session_id: session_id.clone(),
        file_id,
        saved_path,
        text_content,
    });
    if completed_now {
        push_incoming_event(IncomingTransferEvent::Completed {
            session_id: session_id.clone(),
        });
        state.sessions.lock().await.remove(&session_id);
    }

    Ok(json_response(StatusCode::OK, &serde_json::json!({})))
}

async fn handle_cancel(
    req: Request<Incoming>,
    state: ServerState,
    v2: bool,
) -> Result<Response<Full<Bytes>>, (StatusCode, String)> {
    let query = parse_query(req.uri().query().unwrap_or_default());
    let session_id_query = query_first(&query, &["sessionId", "session_id", "sid"]).cloned();
    if v2 && session_id_query.is_none() {
        return Err((StatusCode::BAD_REQUEST, "missing sessionId".to_string()));
    }

    let session_id = if let Some(session_id) = session_id_query {
        session_id
    } else {
        state
            .sessions
            .lock()
            .await
            .keys()
            .next()
            .cloned()
            .unwrap_or_default()
    };

    if !session_id.is_empty() {
        if let Some(session) = state.sessions.lock().await.remove(&session_id) {
            log::info!(
                "cancelled session {} from {} ({:?}), files={}",
                session_id,
                session.sender_alias,
                session.sender_device_model,
                session.files.len()
            );
            push_incoming_event(IncomingTransferEvent::Cancelled {
                session_id,
                reason: Some(format!("cancelled by {}", session.sender_fingerprint)),
            });
        }
    }
    Ok(json_response(StatusCode::OK, &serde_json::json!({})))
}

async fn parse_json_body<T: serde::de::DeserializeOwned>(
    body: Incoming,
) -> Result<T, (StatusCode, String)> {
    let bytes = body
        .collect()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("read body failed: {}", e)))?
        .to_bytes();
    serde_json::from_slice(&bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid json: {}", e)))
}

fn parse_query(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut it = pair.splitn(2, '=');
        let k = it.next().unwrap_or_default();
        let v = it.next().unwrap_or_default();
        map.insert(k.to_string(), v.to_string());
    }
    map
}

fn query_first<'a>(query: &'a HashMap<String, String>, keys: &[&str]) -> Option<&'a String> {
    for key in keys {
        if let Some(value) = query.get(*key) {
            return Some(value);
        }
    }
    None
}

fn sanitize_file_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return format!("{}.bin", uuid::Uuid::new_v4());
    }
    trimmed.replace('/', "_").replace('\\', "_")
}

fn normalize_file_type(ft: &str) -> String {
    if ft.contains('/') {
        ft.to_string()
    } else {
        match ft.to_ascii_lowercase().as_str() {
            "text" => "text/plain".to_string(),
            "image" => "image/*".to_string(),
            "video" => "video/*".to_string(),
            "pdf" => "application/pdf".to_string(),
            "apk" => "application/vnd.android.package-archive".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }
}

fn is_text_type(ft: &str) -> bool {
    let t = ft.to_ascii_lowercase();
    t.starts_with("text/") || t == "text"
}

fn map_device_type_to_wire(
    device_type: Option<&localsend::model::discovery::DeviceType>,
) -> Option<String> {
    match device_type {
        Some(localsend::model::discovery::DeviceType::Mobile) => Some("mobile".to_string()),
        Some(localsend::model::discovery::DeviceType::Desktop) => Some("desktop".to_string()),
        Some(localsend::model::discovery::DeviceType::Web) => Some("web".to_string()),
        Some(localsend::model::discovery::DeviceType::Headless) => Some("headless".to_string()),
        Some(localsend::model::discovery::DeviceType::Server) => Some("server".to_string()),
        None => None,
    }
}

fn map_wire_device_type(value: Option<&str>) -> Option<localsend::model::discovery::DeviceType> {
    match value {
        Some("mobile") | Some("MOBILE") => Some(localsend::model::discovery::DeviceType::Mobile),
        Some("desktop") | Some("DESKTOP") => Some(localsend::model::discovery::DeviceType::Desktop),
        Some("web") | Some("WEB") => Some(localsend::model::discovery::DeviceType::Web),
        Some("headless") | Some("HEADLESS") => {
            Some(localsend::model::discovery::DeviceType::Headless)
        }
        Some("server") | Some("SERVER") => Some(localsend::model::discovery::DeviceType::Server),
        _ => None,
    }
}

fn json_response<T: serde::Serialize>(status: StatusCode, body: &T) -> Response<Full<Bytes>> {
    let mut response = Response::new(Full::new(Bytes::new()));
    *response.status_mut() = status;
    response.headers_mut().insert(
        CONTENT_TYPE,
        hyper::http::HeaderValue::from_static("application/json"),
    );
    let payload = serde_json::to_vec(body).unwrap_or_else(|_| b"{}".to_vec());
    *response.body_mut() = Full::new(Bytes::from(payload));
    response
}

fn no_content() -> Response<Full<Bytes>> {
    let mut response = Response::new(Full::new(Bytes::new()));
    *response.status_mut() = StatusCode::NO_CONTENT;
    response
}
