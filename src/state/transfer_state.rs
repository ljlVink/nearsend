use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Transfer direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferDirection {
    Send,
    Receive,
}

/// Transfer status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    #[allow(dead_code)]
    pub eta_seconds: Option<u64>,
    pub files: Vec<FileTransferInfo>,
}

/// Transfer state management
#[derive(Clone)]
pub struct TransferState {
    transfers: Arc<RwLock<HashMap<String, TransferInfo>>>,
    active_send_id: Arc<RwLock<Option<String>>>,
    active_receive_id: Arc<RwLock<Option<String>>>,
}

impl TransferState {
    pub fn new() -> Self {
        Self {
            transfers: Arc::new(RwLock::new(HashMap::new())),
            active_send_id: Arc::new(RwLock::new(None)),
            active_receive_id: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn add_transfer(&self, transfer: TransferInfo) {
        let id = transfer.id.clone();
        let direction = transfer.direction;
        self.transfers.write().await.insert(id.clone(), transfer);
        match direction {
            TransferDirection::Send => {
                *self.active_send_id.write().await = Some(id);
            }
            TransferDirection::Receive => {
                *self.active_receive_id.write().await = Some(id);
            }
        }
    }

    #[allow(dead_code)]
    pub async fn update_transfer(&self, id: &str, status: TransferStatus, progress: f64) {
        if let Some(transfer) = self.transfers.write().await.get_mut(id) {
            transfer.status = status;
            transfer.progress = progress;
        }
    }

    pub async fn update_transfer_status(&self, id: &str, status: TransferStatus) {
        if let Some(transfer) = self.transfers.write().await.get_mut(id) {
            if transfer.status == TransferStatus::Cancelled && status != TransferStatus::Cancelled {
                return;
            }
            transfer.status = status;
        }
    }

    pub async fn mark_file_in_progress(&self, id: &str, file_id: &str) {
        if let Some(transfer) = self.transfers.write().await.get_mut(id) {
            if transfer.status == TransferStatus::Cancelled {
                return;
            }
            if let Some(file) = transfer.files.iter_mut().find(|f| f.file_id == file_id) {
                file.status = TransferStatus::InProgress;
                transfer.file_name = file.file_name.clone();
            }
            recalculate_transfer_progress(transfer);
        }
    }

    pub async fn mark_file_completed(&self, id: &str, file_id: &str) {
        if let Some(transfer) = self.transfers.write().await.get_mut(id) {
            if transfer.status == TransferStatus::Cancelled {
                return;
            }
            if let Some(file) = transfer.files.iter_mut().find(|f| f.file_id == file_id) {
                file.status = TransferStatus::Completed;
                file.bytes_transferred = file.file_size;
            }
            recalculate_transfer_progress(transfer);
        }
    }

    pub async fn mark_file_failed(&self, id: &str, file_id: &str) {
        if let Some(transfer) = self.transfers.write().await.get_mut(id) {
            if transfer.status == TransferStatus::Cancelled {
                return;
            }
            if let Some(file) = transfer.files.iter_mut().find(|f| f.file_id == file_id) {
                file.status = TransferStatus::Failed;
            }
            recalculate_transfer_progress(transfer);
        }
    }

    pub async fn update_file_progress(
        &self,
        id: &str,
        file_id: &str,
        bytes_transferred: u64,
        speed_bytes_per_sec: u64,
    ) {
        if let Some(transfer) = self.transfers.write().await.get_mut(id) {
            if transfer.status == TransferStatus::Cancelled {
                return;
            }
            if let Some(file) = transfer.files.iter_mut().find(|f| f.file_id == file_id) {
                file.bytes_transferred = bytes_transferred.min(file.file_size);
                if file.status == TransferStatus::Pending {
                    file.status = TransferStatus::InProgress;
                }
                transfer.file_name = file.file_name.clone();
            }
            transfer.speed_bytes_per_sec = speed_bytes_per_sec;
            recalculate_transfer_progress(transfer);
        }
    }

    pub fn snapshot_latest_by_direction(
        &self,
        direction: TransferDirection,
    ) -> Option<TransferInfo> {
        let active_id = match direction {
            TransferDirection::Send => self.active_send_id.try_read().ok()?.clone(),
            TransferDirection::Receive => self.active_receive_id.try_read().ok()?.clone(),
        };
        let guard = self.transfers.try_read().ok()?;
        if let Some(id) = active_id {
            if let Some(active) = guard.get(&id) {
                return Some(active.clone());
            }
        }
        let mut candidate: Option<TransferInfo> = None;
        for transfer in guard.values() {
            if transfer.direction != direction {
                continue;
            }
            if !is_terminal_status(transfer.status) {
                return Some(transfer.clone());
            }
            if candidate.is_none() {
                candidate = Some(transfer.clone());
            }
        }
        candidate
    }

    #[allow(dead_code)]
    pub async fn get_transfers(&self) -> Vec<TransferInfo> {
        self.transfers.read().await.values().cloned().collect()
    }

    #[allow(dead_code)]
    pub async fn remove_transfer(&self, id: &str) {
        let removed = self.transfers.write().await.remove(id);
        if let Some(transfer) = removed {
            match transfer.direction {
                TransferDirection::Send => {
                    let mut active = self.active_send_id.write().await;
                    if active.as_deref() == Some(id) {
                        *active = None;
                    }
                }
                TransferDirection::Receive => {
                    let mut active = self.active_receive_id.write().await;
                    if active.as_deref() == Some(id) {
                        *active = None;
                    }
                }
            }
        }
    }
}

impl Default for TransferState {
    fn default() -> Self {
        Self::new()
    }
}

fn recalculate_transfer_progress(transfer: &mut TransferInfo) {
    transfer.bytes_sent = transfer.files.iter().map(|f| f.bytes_transferred).sum();
    transfer.total_bytes = transfer.files.iter().map(|f| f.file_size).sum();
    transfer.progress = if transfer.total_bytes > 0 {
        transfer.bytes_sent as f64 / transfer.total_bytes as f64
    } else {
        0.0
    };

    if transfer
        .files
        .iter()
        .all(|f| matches!(f.status, TransferStatus::Completed))
    {
        transfer.status = TransferStatus::Completed;
    } else if transfer
        .files
        .iter()
        .any(|f| matches!(f.status, TransferStatus::Failed))
    {
        transfer.status = TransferStatus::Failed;
    } else if transfer
        .files
        .iter()
        .any(|f| matches!(f.status, TransferStatus::InProgress))
    {
        transfer.status = TransferStatus::InProgress;
    } else {
        transfer.status = TransferStatus::Pending;
    }
}

fn is_terminal_status(status: TransferStatus) -> bool {
    matches!(
        status,
        TransferStatus::Completed | TransferStatus::Failed | TransferStatus::Cancelled
    )
}
