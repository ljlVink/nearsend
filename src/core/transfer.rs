use localsend::http::client::LsHttpClient;
use localsend::http::dto::{PrepareUploadRequestDto, ProtocolType, RegisterDto};
use localsend::http::state::ClientInfo;
use std::path::PathBuf;

/// File transfer service using LocalSend core
pub struct TransferService {
    client: Option<LsHttpClient>,
}

impl TransferService {
    pub fn new() -> Self {
        Self { client: None }
    }

    /// Initialize transfer client
    pub async fn init(&mut self, private_key: &str, cert: &str) -> anyhow::Result<()> {
        // TODO: Generate TLS certificate using localsend::crypto
        self.client = Some(LsHttpClient::try_new(private_key, cert)?);
        Ok(())
    }

    /// Send files to a device
    pub async fn send_files(
        &self,
        device_info: &ClientInfo,
        ip: &str,
        port: u16,
        protocol: ProtocolType,
        files: Vec<PathBuf>,
    ) -> anyhow::Result<String> {
        // TODO: Implement file sending using localsend client
        // 1. Exchange nonce
        // 2. Register
        // 3. Prepare upload
        // 4. Upload files
        log::info!(
            "Sending {} files to device {}",
            files.len(),
            device_info.alias
        );
        Ok("transfer_id".to_string())
    }
}

impl Default for TransferService {
    fn default() -> Self {
        Self::new()
    }
}
