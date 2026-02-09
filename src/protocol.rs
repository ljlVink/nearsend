use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LocalSend Protocol version
pub const PROTOCOL_VERSION: &str = "2.0";

/// Default multicast address
pub const MULTICAST_ADDRESS: &str = "224.0.0.167";

/// Default multicast port
pub const MULTICAST_PORT: u16 = 53317;

/// Device types supported by LocalSend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Mobile,
    Desktop,
    Web,
    Headless,
    Server,
}

/// Device information sent in discovery announcements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub alias: String,
    pub version: String,
    pub device_model: String,
    pub device_type: DeviceType,
    pub fingerprint: String,
    pub port: u16,
    pub protocol: String, // "http" or "https"
    pub download: bool,
    #[serde(skip)]
    pub ip_address: Option<String>, // IP address discovered from multicast packet
}

/// File information for transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_type: Option<String>,
}

/// Message information for transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInfo {
    pub text: String,
}

/// Transfer request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub files: Vec<FileInfo>,
    pub text: Option<String>,
}

/// Transfer response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResponse {
    pub transfer_id: String,
    pub files: Vec<FileInfo>,
    pub text: Option<String>,
}

/// Action request for file transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    pub action: String, // "accept" or "reject"
    pub transfer_id: String,
}

/// Action response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResponse {
    pub status: String, // "success" or "error"
    pub message: Option<String>,
}

/// Transfer status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferStatus {
    Pending,
    Accepted,
    Rejected,
    InProgress,
    Completed,
    Failed,
}

/// Transfer progress information
#[derive(Debug, Clone)]
pub struct TransferProgress {
    pub transfer_id: String,
    pub status: TransferStatus,
    pub bytes_sent: u64,
    pub total_bytes: u64,
    pub file_index: usize,
    pub file_name: String,
}

impl TransferProgress {
    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.bytes_sent as f64 / self.total_bytes as f64) * 100.0
        }
    }
}
