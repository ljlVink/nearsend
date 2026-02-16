use crate::state::transfer_state::{TransferDirection, TransferStatus};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoryEntryKind {
    #[default]
    File,
    Text,
}

/// A single history entry for a completed (or failed) transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_path: PathBuf,
    #[serde(default)]
    pub kind: HistoryEntryKind,
    #[serde(default)]
    pub text_content: Option<String>,
    pub direction: TransferDirection,
    pub device_name: String,
    pub timestamp: u64, // unix timestamp in seconds
    pub status: TransferStatus,
}

/// Manages transfer history entries.
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct HistoryState {
    entries: Vec<HistoryEntry>,
}

impl HistoryState {
    pub fn new() -> Self {
        Self::load_from_disk()
    }

    pub fn add_entry(&mut self, entry: HistoryEntry) {
        self.entries.insert(0, entry); // newest first
        self.persist_to_disk();
    }

    pub fn remove_entry(&mut self, id: &str) {
        self.entries.retain(|e| e.id != id);
        self.persist_to_disk();
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.persist_to_disk();
    }

    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    fn load_from_disk() -> Self {
        let path = crate::platform::preferences_path::get_preferences_file_path("history.json");
        let Ok(raw) = std::fs::read_to_string(&path) else {
            return Self::default();
        };

        match serde_json::from_str::<Self>(&raw) {
            Ok(state) => state,
            Err(err) => {
                log::warn!("failed to parse history file {}: {}", path.display(), err);
                Self::default()
            }
        }
    }

    fn persist_to_disk(&self) {
        let path = crate::platform::preferences_path::get_preferences_file_path("history.json");
        if let Some(dir) = path.parent() {
            if let Err(err) = std::fs::create_dir_all(dir) {
                log::warn!(
                    "failed to create preferences dir {}: {}",
                    dir.display(),
                    err
                );
                return;
            }
        }

        let Ok(serialized) = serde_json::to_string_pretty(self) else {
            log::warn!("failed to serialize history state");
            return;
        };
        if let Err(err) = std::fs::write(&path, serialized) {
            log::warn!("failed to write history file {}: {}", path.display(), err);
        }
    }
}

impl Default for HistoryState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}
