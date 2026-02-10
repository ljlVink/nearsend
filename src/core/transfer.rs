use localsend::http::client::LsHttpClient;
use localsend::http::dto::{PrepareUploadRequestDto, ProtocolType, RegisterDto};
use localsend::model::transfer::FileDto;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;

/// What to send: either text content or a file on disk.
pub enum SendFileEntry {
    Text {
        content: String,
    },
    File {
        path: PathBuf,
        name: String,
        size: u64,
        file_type: String,
    },
}

/// File transfer service using LocalSend core
pub struct TransferService {
    client: Option<LsHttpClient>,
}

impl TransferService {
    pub fn new() -> Self {
        Self { client: None }
    }

    /// Initialize transfer client (sync — just stores the client).
    pub fn init_sync(&mut self, private_key: &str, cert: &str) {
        match LsHttpClient::try_new(private_key, cert) {
            Ok(client) => {
                self.client = Some(client);
                log::info!("Transfer client initialized");
            }
            Err(e) => {
                log::error!("Failed to initialize transfer client: {}", e);
            }
        }
    }

    /// Send files/text to a device using the LocalSend protocol.
    ///
    /// Flow: prepare-upload → for each accepted file: upload bytes via channel.
    pub async fn send(
        &self,
        ip: &str,
        port: u16,
        files: Vec<SendFileEntry>,
        our_info: RegisterDto,
    ) -> anyhow::Result<String> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Transfer client not initialized"))?;

        let protocol = ProtocolType::Http;

        // Build FileDto map and keep entries for later
        let mut file_map: HashMap<String, FileDto> = HashMap::new();
        let mut entry_map: HashMap<String, SendFileEntry> = HashMap::new();
        for entry in files {
            let file_id = uuid::Uuid::new_v4().to_string();
            let dto = match &entry {
                SendFileEntry::Text { content } => FileDto {
                    id: file_id.clone(),
                    file_name: "text.txt".to_string(),
                    size: content.as_bytes().len() as u64,
                    file_type: "text/plain".to_string(),
                    sha256: None,
                    preview: None,
                    metadata: None,
                },
                SendFileEntry::File {
                    path: _,
                    name,
                    size,
                    file_type,
                } => FileDto {
                    id: file_id.clone(),
                    file_name: name.clone(),
                    size: *size,
                    file_type: file_type.clone(),
                    sha256: None,
                    preview: None,
                    metadata: None,
                },
            };
            file_map.insert(file_id.clone(), dto);
            entry_map.insert(file_id, entry);
        }

        let payload = PrepareUploadRequestDto {
            info: our_info,
            files: file_map,
        };

        log::info!("Sending prepare-upload to {}:{}", ip, port);
        let response = client
            .prepare_upload(&protocol, ip, port, None, payload)
            .await?;
        let session_id = response.session_id.clone();
        log::info!(
            "Got session_id: {}, accepted files: {}",
            session_id,
            response.files.len()
        );

        // Upload each accepted file sequentially
        for (file_id, token) in &response.files {
            if let Some(entry) = entry_map.remove(file_id) {
                let (tx, rx) = mpsc::channel::<Vec<u8>>(32);

                // Send data through the channel first (in a spawned task)
                let data_task = tokio::task::spawn(async move {
                    match entry {
                        SendFileEntry::Text { content } => {
                            let _ = tx.send(content.into_bytes()).await;
                        }
                        SendFileEntry::File { path, .. } => match tokio::fs::read(&path).await {
                            Ok(data) => {
                                let _ = tx.send(data).await;
                            }
                            Err(e) => log::error!("Failed to read file {:?}: {}", path, e),
                        },
                    }
                    // tx drops here, closing the channel
                });

                // Upload reads from rx
                log::info!("Uploading file_id={}", file_id);
                let result = client
                    .upload(
                        &protocol,
                        ip,
                        port,
                        session_id.clone(),
                        file_id.clone(),
                        token.clone(),
                        rx,
                    )
                    .await;

                if let Err(e) = result {
                    log::error!("Upload failed for file_id={}: {}", file_id, e);
                }

                let _ = data_task.await;
            }
        }

        log::info!("Transfer complete, session_id={}", session_id);
        Ok(session_id)
    }
}

impl Default for TransferService {
    fn default() -> Self {
        Self::new()
    }
}
