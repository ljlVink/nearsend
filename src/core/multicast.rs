use localsend::http::state::ClientInfo;
use serde::{Deserialize, Serialize};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

const DEFAULT_MULTICAST_GROUP: &str = "224.0.0.167";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ProtocolType {
    Http,
    Https,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum DeviceType {
    Mobile,
    Desktop,
    Web,
    Headless,
    Server,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MulticastDto {
    alias: String,
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_type: Option<DeviceType>,
    fingerprint: String,
    port: Option<u16>,
    protocol: Option<ProtocolType>,
    download: Option<bool>,
    announcement: Option<bool>,
    announce: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IncomingMulticastDto {
    alias: String,
    version: Option<String>,
    #[serde(default)]
    device_model: Option<String>,
    #[serde(default)]
    device_type: Option<String>,
    fingerprint: String,
    #[serde(default)]
    port: Option<u16>,
    #[serde(default)]
    protocol: Option<String>,
    #[serde(default)]
    download: Option<bool>,
    #[serde(default)]
    announcement: Option<bool>,
    #[serde(default)]
    announce: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RegisterDto {
    alias: String,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_type: Option<DeviceType>,
    fingerprint: String,
    port: u16,
    protocol: ProtocolType,
    download: bool,
}

pub async fn start_multicast_service(
    multicast_group: String,
    alias: String,
    fingerprint: String,
    port: u16,
    use_https: bool,
    device_model: Option<String>,
    device_type: Option<localsend::model::discovery::DeviceType>,
) -> anyhow::Result<()> {
    let group = parse_multicast_group(&multicast_group)?;
    let sockets = create_listener_sockets(group, port).await?;
    log::info!("multicast listeners active: {}", sockets.len());
    let (packet_tx, mut packet_rx) = tokio::sync::mpsc::channel::<(Vec<u8>, SocketAddr)>(256);
    for socket in sockets {
        let tx = packet_tx.clone();
        tokio::spawn(async move {
            let mut buf = vec![0u8; 64 * 1024];
            loop {
                let (size, addr) = match socket.recv_from(&mut buf).await {
                    Ok(v) => v,
                    Err(err) => {
                        log::warn!("multicast recv error: {}", err);
                        break;
                    }
                };
                log::debug!("multicast packet received from {} bytes={}", addr, size);
                let data = buf[..size].to_vec();
                if tx.send((data, addr)).await.is_err() {
                    break;
                }
            }
        });
    }
    drop(packet_tx);

    send_multicast_announcement(
        multicast_group.clone(),
        alias.clone(),
        fingerprint.clone(),
        port,
        use_https,
        device_model.clone(),
        device_type.clone(),
    )
    .await?;

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_millis(1200))
        .build()?;
    loop {
        let (payload, addr) = match packet_rx.recv().await {
            Some(v) => v,
            None => {
                return Err(anyhow::anyhow!(
                    "all multicast listeners terminated unexpectedly"
                ));
            }
        };
        let dto = match parse_multicast_payload(&payload) {
            Ok(dto) => dto,
            Err(err) => {
                log::debug!("ignore invalid multicast payload from {}: {}", addr, err);
                continue;
            }
        };
        if dto.fingerprint == fingerprint {
            continue;
        }

        let peer_alias = if dto.alias.trim().is_empty() {
            addr.ip().to_string()
        } else {
            dto.alias.clone()
        };
        let peer_ip = addr.ip().to_string();
        let peer_port = dto.port.unwrap_or(port);
        let peer_https = dto
            .protocol
            .as_ref()
            .map(|p| matches!(p, ProtocolType::Https))
            .unwrap_or(use_https);
        log::info!(
            "multicast discovered peer alias={} ip={} port={} https={}",
            peer_alias,
            peer_ip,
            peer_port,
            peer_https
        );
        crate::core::discovery::register_passive_device(crate::core::discovery::DiscoveredDevice {
            info: ClientInfo {
                alias: peer_alias.clone(),
                version: dto.version.clone().unwrap_or_else(|| "1.0".to_string()),
                device_model: dto.device_model.clone(),
                device_type: map_device_type_to_core(dto.device_type.as_ref()),
                token: dto.fingerprint.clone(),
            },
            ip: peer_ip.clone(),
            port: peer_port,
            https: peer_https,
        });

        // Align with LocalSend: only announcement packets should trigger a register response.
        // Non-announcement packets are still valid discovery signals and must not be dropped.
        if !(dto.announce.unwrap_or(false) || dto.announcement.unwrap_or(false)) {
            continue;
        }

        let register = RegisterDto {
            alias: alias.clone(),
            version: "2.1".to_string(),
            device_model: device_model.clone(),
            device_type: map_device_type(device_type.as_ref()),
            fingerprint: fingerprint.clone(),
            port,
            protocol: if use_https {
                ProtocolType::Https
            } else {
                ProtocolType::Http
            },
            download: false,
        };

        let scheme = match dto.protocol.as_ref() {
            Some(ProtocolType::Https) => "https",
            Some(ProtocolType::Http) => "http",
            None => {
                if use_https {
                    "https"
                } else {
                    "http"
                }
            }
        };
        let register_path = if dto.version.as_deref().unwrap_or("1.0") == "1.0" {
            "/api/localsend/v1/register"
        } else {
            "/api/localsend/v2/register"
        };
        let url = format!("{}://{}:{}{}", scheme, peer_ip, peer_port, register_path);

        if let Err(err) = client.post(&url).json(&register).send().await {
            log::debug!("multicast register response failed {}: {}", url, err);
            if let Err(fallback_err) = send_multicast_presence(
                multicast_group.clone(),
                alias.clone(),
                fingerprint.clone(),
                port,
                use_https,
                device_model.clone(),
                device_type.clone(),
                false,
            )
            .await
            {
                log::debug!("multicast udp fallback response failed: {}", fallback_err);
            }
        }
    }
}

async fn create_listener_sockets(
    group: Ipv4Addr,
    port: u16,
) -> anyhow::Result<Vec<tokio::net::UdpSocket>> {
    let interfaces = local_ipv4_interfaces();
    log::info!("multicast join candidate interfaces: {:?}", interfaces);
    let mut sockets = Vec::new();
    for iface in interfaces {
        let socket = tokio::net::UdpSocket::from_std(bind_reuse_udp_socket(port)?.into())?;
        if let Err(err) = socket.join_multicast_v4(group, iface) {
            log::debug!(
                "join multicast failed for iface {} group {}: {}",
                iface,
                group,
                err
            );
            continue;
        }
        log::info!("joined multicast group {} on iface {}", group, iface);
        sockets.push(socket);
    }

    if sockets.is_empty() {
        let socket = tokio::net::UdpSocket::from_std(bind_reuse_udp_socket(port)?.into())?;
        socket.join_multicast_v4(group, Ipv4Addr::UNSPECIFIED)?;
        log::info!(
            "joined multicast group {} on unspecified iface (fallback)",
            group
        );
        sockets.push(socket);
    }

    for socket in &sockets {
        if let Err(err) = socket.set_broadcast(true) {
            log::debug!("set broadcast failed on 0.0.0.0:{}: {}", port, err);
        }
    }

    Ok(sockets)
}

pub async fn send_multicast_announcement(
    multicast_group: String,
    alias: String,
    fingerprint: String,
    port: u16,
    use_https: bool,
    device_model: Option<String>,
    device_type: Option<localsend::model::discovery::DeviceType>,
) -> anyhow::Result<()> {
    send_multicast_presence(
        multicast_group,
        alias,
        fingerprint,
        port,
        use_https,
        device_model,
        device_type,
        true,
    )
    .await
}

async fn send_multicast_presence(
    multicast_group: String,
    alias: String,
    fingerprint: String,
    port: u16,
    use_https: bool,
    device_model: Option<String>,
    device_type: Option<localsend::model::discovery::DeviceType>,
    announcement: bool,
) -> anyhow::Result<()> {
    let info = MulticastDto {
        alias,
        version: Some("2.1".to_string()),
        device_model,
        device_type: map_device_type(device_type.as_ref()),
        fingerprint,
        port: Some(port),
        protocol: Some(if use_https {
            ProtocolType::Https
        } else {
            ProtocolType::Http
        }),
        download: Some(false),
        announcement: Some(announcement),
        announce: Some(announcement),
    };

    let payload = serde_json::to_vec(&info)?;
    let group = parse_multicast_group(&multicast_group)?;
    let destination = SocketAddrV4::new(group, port);
    let delays: &[u64] = if announcement {
        &[100, 500, 2000]
    } else {
        &[0]
    };
    for &delay_ms in delays {
        if delay_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        }
        let mut sent_any = false;
        for iface in local_ipv4_interfaces() {
            let socket = match bind_multicast_sender_socket(iface) {
                Ok(socket) => socket,
                Err(err) => {
                    log::debug!(
                        "announcement sender setup failed on iface {}: {}",
                        iface,
                        err
                    );
                    continue;
                }
            };
            if let Err(err) = socket.send_to(&payload, destination).await {
                log::debug!(
                    "multicast announcement send failed on iface {}: {}",
                    iface,
                    err
                );
            } else {
                sent_any = true;
            }
        }
        if !sent_any {
            let socket = tokio::net::UdpSocket::bind(("0.0.0.0", 0)).await?;
            if let Err(err) = socket.send_to(&payload, destination).await {
                log::debug!("multicast announcement send failed: {}", err);
            }
        }
    }

    Ok(())
}

fn bind_reuse_udp_socket(port: u16) -> anyhow::Result<Socket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.bind(&std::net::SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port).into())?;
    socket.set_nonblocking(true)?;
    Ok(socket)
}

fn bind_multicast_sender_socket(iface: Ipv4Addr) -> anyhow::Result<tokio::net::UdpSocket> {
    let socket = bind_reuse_udp_socket(0)?;
    socket.set_multicast_if_v4(&iface)?;
    socket.set_multicast_loop_v4(true)?;
    socket.set_multicast_ttl_v4(1)?;
    Ok(tokio::net::UdpSocket::from_std(socket.into())?)
}

fn parse_multicast_group(group: &str) -> anyhow::Result<Ipv4Addr> {
    let raw = if group.trim().is_empty() {
        DEFAULT_MULTICAST_GROUP
    } else {
        group.trim()
    };
    let addr: Ipv4Addr = raw
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid multicast group: {}", raw))?;
    if !addr.is_multicast() {
        anyhow::bail!("multicast group is not in 224.0.0.0/4: {}", raw);
    }
    Ok(addr)
}

fn parse_multicast_payload(payload: &[u8]) -> anyhow::Result<MulticastDto> {
    let incoming: IncomingMulticastDto = serde_json::from_slice(payload)?;
    Ok(MulticastDto {
        alias: incoming.alias,
        version: incoming.version,
        device_model: incoming.device_model,
        device_type: map_device_type_incoming(incoming.device_type.as_deref()),
        fingerprint: incoming.fingerprint,
        port: incoming.port,
        protocol: map_protocol_incoming(incoming.protocol.as_deref()),
        download: incoming.download,
        announcement: incoming.announcement,
        announce: incoming.announce,
    })
}

fn local_ipv4_interfaces() -> Vec<Ipv4Addr> {
    let mut out = Vec::<Ipv4Addr>::new();
    let interfaces = match if_addrs::get_if_addrs() {
        Ok(v) => v,
        Err(err) => {
            log::warn!("failed to enumerate interfaces for multicast join: {}", err);
            return out;
        }
    };
    for iface in interfaces {
        if iface.is_loopback() {
            continue;
        }
        if let if_addrs::IfAddr::V4(v4) = iface.addr {
            if v4.ip.is_link_local() {
                continue;
            }
            out.push(v4.ip);
        }
    }
    rank_ipv4_addresses(&mut out, detect_primary_route_ipv4());
    out.dedup();
    out
}

fn detect_primary_route_ipv4() -> Option<Ipv4Addr> {
    let probes = [("224.0.0.167", 53317), ("1.1.1.1", 80), ("8.8.8.8", 80)];
    for (host, port) in probes {
        let socket = match std::net::UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(_) => continue,
        };
        if socket.connect((host, port)).is_err() {
            continue;
        }
        let local = match socket.local_addr() {
            Ok(addr) => addr,
            Err(_) => continue,
        };
        if let std::net::IpAddr::V4(ipv4) = local.ip() {
            return Some(ipv4);
        }
    }
    None
}

fn rank_ipv4_addresses(list: &mut Vec<Ipv4Addr>, primary: Option<Ipv4Addr>) {
    list.sort_by(|a, b| {
        let score = |ip: &Ipv4Addr| -> i32 {
            if Some(*ip) == primary {
                10
            } else if ip.octets()[3] == 1 {
                0
            } else {
                1
            }
        };
        score(b)
            .cmp(&score(a))
            .then_with(|| a.octets().cmp(&b.octets()))
    });
}

fn map_device_type(value: Option<&localsend::model::discovery::DeviceType>) -> Option<DeviceType> {
    match value {
        Some(localsend::model::discovery::DeviceType::Mobile) => Some(DeviceType::Mobile),
        Some(localsend::model::discovery::DeviceType::Desktop) => Some(DeviceType::Desktop),
        Some(localsend::model::discovery::DeviceType::Web) => Some(DeviceType::Web),
        Some(localsend::model::discovery::DeviceType::Headless) => Some(DeviceType::Headless),
        Some(localsend::model::discovery::DeviceType::Server) => Some(DeviceType::Server),
        None => None,
    }
}

fn map_device_type_incoming(value: Option<&str>) -> Option<DeviceType> {
    match value {
        Some("mobile") | Some("MOBILE") => Some(DeviceType::Mobile),
        Some("desktop") | Some("DESKTOP") => Some(DeviceType::Desktop),
        Some("web") | Some("WEB") => Some(DeviceType::Web),
        Some("headless") | Some("HEADLESS") => Some(DeviceType::Headless),
        Some("server") | Some("SERVER") => Some(DeviceType::Server),
        Some(_) => Some(DeviceType::Desktop),
        None => None,
    }
}

fn map_protocol_incoming(value: Option<&str>) -> Option<ProtocolType> {
    match value {
        Some("http") | Some("HTTP") => Some(ProtocolType::Http),
        Some("https") | Some("HTTPS") => Some(ProtocolType::Https),
        Some(_) => Some(ProtocolType::Https),
        None => None,
    }
}

fn map_device_type_to_core(
    value: Option<&DeviceType>,
) -> Option<localsend::model::discovery::DeviceType> {
    match value {
        Some(DeviceType::Mobile) => Some(localsend::model::discovery::DeviceType::Mobile),
        Some(DeviceType::Desktop) => Some(localsend::model::discovery::DeviceType::Desktop),
        Some(DeviceType::Web) => Some(localsend::model::discovery::DeviceType::Web),
        Some(DeviceType::Headless) => Some(localsend::model::discovery::DeviceType::Headless),
        Some(DeviceType::Server) => Some(localsend::model::discovery::DeviceType::Server),
        None => None,
    }
}
