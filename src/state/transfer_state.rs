use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Transfer direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferDirection {
    Send,
    Receive,
}

/// Transfer status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Skipped,
}

/// Per-file transfer progress
#[derive(Debug, Clone)]
pub struct FileTransferInfo {
    pub file_id: String,
    pub file_name: String,
    pub file_size: u64,
    pub bytes_transferred: u64,
    pub status: TransferStatus,
}

/// Transfer information
#[derive(Clone)]
pub struct TransferInfo {
    pub id: String,
    pub device_name: String,
    pub status: TransferStatus,
    pub direction: TransferDirection,
    pub progress: f64, // 0.0 to 1.0
    pub bytes_sent: u64,
    pub total_bytes: u64,
    pub file_name: String,
    pub speed_bytes_per_sec: u64,
    pub eta_seconds: Option<u64>,
    pub files: Vec<FileTransferInfo>,
}

/// Transfer state management
#[derive(Clone)]
pub struct TransferState {
    transfers: Arc<RwLock<HashMap<String, TransferInfo>>>,
}

impl TransferState {
    pub fn new() -> Self {
        Self {
            transfers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_transfer(&self, transfer: TransferInfo) {
        self.transfers
            .write()
            .await
            .insert(transfer.id.clone(), transfer);
    }

    pub async fn update_transfer(&self, id: &str, status: TransferStatus, progress: f64) {
        if let Some(transfer) = self.transfers.write().await.get_mut(id) {
            transfer.status = status;
            transfer.progress = progress;
        }
    }

    pub async fn get_transfers(&self) -> Vec<TransferInfo> {
        self.transfers.read().await.values().cloned().collect()
    }

    pub async fn remove_transfer(&self, id: &str) {
        self.transfers.write().await.remove(id);
    }
}

impl Default for TransferState {
    fn default() -> Self {
        Self::new()
    }
}
