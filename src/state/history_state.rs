use crate::state::transfer_state::{TransferDirection, TransferStatus};
use std::path::PathBuf;

/// A single history entry for a completed (or failed) transfer.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_path: PathBuf,
    pub direction: TransferDirection,
    pub device_name: String,
    pub timestamp: u64, // unix timestamp in seconds
    pub status: TransferStatus,
}

/// Manages transfer history entries.
pub struct HistoryState {
    entries: Vec<HistoryEntry>,
}

impl HistoryState {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: HistoryEntry) {
        self.entries.insert(0, entry); // newest first
    }

    pub fn remove_entry(&mut self, id: &str) {
        self.entries.retain(|e| e.id != id);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }
}

impl Default for HistoryState {
    fn default() -> Self {
        Self::new()
    }
}
