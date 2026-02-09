use crate::protocol::{ActionRequest, ActionResponse, DeviceInfo, TransferRequest, TransferResponse};
use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

/// HTTP client for sending files and messages
pub struct LocalSendClient {
    client: Client,
    device_info: DeviceInfo,
}

impl LocalSendClient {
    /// Create a new client
    pub fn new(device_info: DeviceInfo) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .danger_accept_invalid_certs(true) // Accept self-signed certificates
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, device_info })
    }

    /// Get base URL for a device
    fn base_url(&self, device: &DeviceInfo) -> String {
        let ip = device.ip_address.as_deref().unwrap_or("127.0.0.1");
        format!("{}://{}:{}", device.protocol, ip, device.port)
    }

    /// Send transfer request to a device
    pub async fn send_transfer(
        &self,
        device: &DeviceInfo,
        request: TransferRequest,
    ) -> Result<TransferResponse> {
        let url = format!("{}/api/transfer", self.base_url(device));
        
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send transfer request")?;

        if !response.status().is_success() {
            anyhow::bail!("Transfer request failed: {}", response.status());
        }

        let transfer_response: TransferResponse = response
            .json()
            .await
            .context("Failed to parse transfer response")?;

        Ok(transfer_response)
    }

    /// Accept or reject a transfer
    pub async fn send_action(
        &self,
        device: &DeviceInfo,
        transfer_id: &str,
        action: &str,
    ) -> Result<ActionResponse> {
        let url = format!("{}/api/transfer/{}", self.base_url(device), transfer_id);
        
        let request = ActionRequest {
            action: action.to_string(),
            transfer_id: transfer_id.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send action")?;

        if !response.status().is_success() {
            anyhow::bail!("Action request failed: {}", response.status());
        }

        let action_response: ActionResponse = response
            .json()
            .await
            .context("Failed to parse action response")?;

        Ok(action_response)
    }

    /// Upload a file
    pub async fn upload_file(
        &self,
        device: &DeviceInfo,
        transfer_id: &str,
        file_id: &str,
        file_data: Vec<u8>,
        file_name: &str,
    ) -> Result<()> {
        let url = format!(
            "{}/api/transfer/{}/file/{}",
            self.base_url(device),
            transfer_id,
            file_id
        );

        let form = reqwest::multipart::Form::new()
            .text("transfer_id", transfer_id.to_string())
            .text("file_id", file_id.to_string())
            .part(
                "file",
                reqwest::multipart::Part::bytes(file_data)
                    .file_name(file_name.to_string())
                    .mime_str("application/octet-stream")?,
            );

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .context("Failed to upload file")?;

        if !response.status().is_success() {
            anyhow::bail!("File upload failed: {}", response.status());
        }

        Ok(())
    }
}

/// Get local device IP address
fn get_device_ip() -> String {
    // Try to get the local IP address
    // This is a simplified version - in production, you'd want to detect
    // the actual network interface IP
    if let Ok(Ok(addr)) = std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| socket.connect("8.8.8.8:80"))
        .and_then(|socket| socket.local_addr())
    {
        return addr.ip().to_string();
    }
    "127.0.0.1".to_string()
}
