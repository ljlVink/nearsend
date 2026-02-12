//! Send tab state and types (selected files, nearby devices, send mode, etc.).

use localsend::http::state::ClientInfo;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Send content type: 文件 / 媒体 / 文本 / 文件夹
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SendContentType {
    #[default]
    File,
    Media,
    Text,
    Folder,
}

/// Send mode (Single / Multiple / Link)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SendMode {
    Single,
    Multiple,
    Link,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SendSessionStatus {
    Idle,
    Preparing,
    PinRequired,
    Sending,
    Completed,
    Declined,
    RecipientBusy,
    TooManyAttempts,
    CancelledByUser,
    Failed,
}

/// Information about a selected file.
#[derive(Clone, Debug)]
pub struct SelectedFileInfo {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub file_type: String, // e.g. "image/png", "application/pdf", or extension
    pub text_content: Option<String>, // For text messages
}

#[derive(Clone, Debug)]
pub struct DeviceEndpoint {
    pub ip: String,
    pub port: u16,
    pub https: bool,
}

/// Send tab state
pub struct SendPageState {
    pub selected_files: Vec<SelectedFileInfo>,
    pub selected_files_total_size: u64,
    pub send_content_type: SendContentType,
    pub nearby_devices: Vec<ClientInfo>,
    pub nearby_endpoints: HashMap<String, DeviceEndpoint>, // key: token(fingerprint)
    pub scanning: bool,
    pub local_ips: Vec<String>,
    pub send_mode: SendMode,
    pub help_index: usize,
    pub show_scan_menu: bool,
    pub show_send_mode_menu: bool,
    pub target_device: Option<ClientInfo>,
    pub target_ip: Option<String>,
    pub pending_send: bool,
    pub session_status: SendSessionStatus,
    pub session_status_text: Option<String>,
    pub has_scanned_once: bool,
    pub favorite_tokens: HashSet<String>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct FavoritesData {
    favorite_tokens: Vec<String>,
}

fn load_favorite_tokens() -> HashSet<String> {
    let path = crate::platform::preferences_path::get_preferences_file_path("favorites.json");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return HashSet::new();
    };
    match serde_json::from_str::<FavoritesData>(&raw) {
        Ok(data) => data.favorite_tokens.into_iter().collect(),
        Err(err) => {
            log::warn!("failed to parse favorites file {}: {}", path.display(), err);
            HashSet::new()
        }
    }
}

impl Default for SendPageState {
    fn default() -> Self {
        Self {
            selected_files: Vec::new(),
            selected_files_total_size: 0,
            send_content_type: SendContentType::default(),
            nearby_devices: Vec::new(),
            nearby_endpoints: HashMap::new(),
            scanning: false,
            local_ips: Vec::new(),
            send_mode: SendMode::Single,
            help_index: 0,
            show_scan_menu: false,
            show_send_mode_menu: false,
            target_device: None,
            target_ip: None,
            pending_send: false,
            session_status: SendSessionStatus::Idle,
            session_status_text: None,
            has_scanned_once: false,
            favorite_tokens: load_favorite_tokens(),
        }
    }
}

impl SendPageState {
    pub fn toggle_favorite_token(&mut self, token: &str) {
        if !self.favorite_tokens.insert(token.to_string()) {
            self.favorite_tokens.remove(token);
        }
        self.persist_favorites();
    }

    fn persist_favorites(&self) {
        let path = crate::platform::preferences_path::get_preferences_file_path("favorites.json");
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

        let mut tokens: Vec<String> = self.favorite_tokens.iter().cloned().collect();
        tokens.sort_unstable();
        let payload = FavoritesData {
            favorite_tokens: tokens,
        };
        let Ok(serialized) = serde_json::to_string_pretty(&payload) else {
            log::warn!("failed to serialize favorites state");
            return;
        };
        if let Err(err) = std::fs::write(&path, serialized) {
            log::warn!("failed to write favorites file {}: {}", path.display(), err);
        }
    }
}
