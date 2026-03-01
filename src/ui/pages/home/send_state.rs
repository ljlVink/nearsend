//! Send tab state and types (selected files, nearby devices, send mode, etc.).

use localsend::http::state::ClientInfo;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

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
    pub source_uri: Option<String>,
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

#[derive(Clone, Debug)]
pub struct ActiveSendContext {
    pub ip: String,
    pub port: u16,
    pub scheme: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FavoriteDevice {
    pub token: String,
    pub alias: String,
    pub ip: String,
    pub port: u16,
    #[serde(default)]
    pub https: bool,
    #[serde(default)]
    pub custom_alias: bool,
}

struct LoadedFavorites {
    tokens: HashSet<String>,
    devices: HashMap<String, FavoriteDevice>,
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
    #[allow(dead_code)]
    pub help_index: usize,
    #[allow(dead_code)]
    pub show_scan_menu: bool,
    pub show_send_mode_menu: bool,
    pub target_device: Option<ClientInfo>,
    pub target_ip: Option<String>,
    pub pending_send: bool,
    pub session_status: SendSessionStatus,
    pub session_status_text: Option<String>,
    pub last_send_ip: Option<String>,
    pub last_send_port: Option<u16>,
    pub active_send_cancel_flag: Option<Arc<AtomicBool>>,
    pub active_send_context: Option<Arc<Mutex<ActiveSendContext>>>,
    pub active_transfer_id: Option<String>,
    pub has_scanned_once: bool,
    pub suppress_next_nearby_row_click: bool,
    pub favorite_tokens: HashSet<String>,
    pub favorite_devices: HashMap<String, FavoriteDevice>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct FavoritesData {
    favorite_tokens: Vec<String>,
    favorite_devices: Vec<FavoriteDevice>,
}

fn load_favorites() -> LoadedFavorites {
    let path = crate::platform::preferences_path::get_preferences_file_path("favorites.json");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return LoadedFavorites {
            tokens: HashSet::new(),
            devices: HashMap::new(),
        };
    };
    match serde_json::from_str::<FavoritesData>(&raw) {
        Ok(data) => {
            let mut tokens: HashSet<String> = data.favorite_tokens.into_iter().collect();
            let mut devices = HashMap::new();
            for item in data.favorite_devices {
                if item.token.trim().is_empty() || item.ip.trim().is_empty() || item.port == 0 {
                    continue;
                }
                tokens.insert(item.token.clone());
                devices.insert(item.token.clone(), item);
            }
            LoadedFavorites { tokens, devices }
        }
        Err(err) => {
            log::warn!("failed to parse favorites file {}: {}", path.display(), err);
            LoadedFavorites {
                tokens: HashSet::new(),
                devices: HashMap::new(),
            }
        }
    }
}

impl Default for SendPageState {
    fn default() -> Self {
        let loaded_favorites = load_favorites();
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
            last_send_ip: None,
            last_send_port: None,
            active_send_cancel_flag: None,
            active_send_context: None,
            active_transfer_id: None,
            has_scanned_once: false,
            suppress_next_nearby_row_click: false,
            favorite_tokens: loaded_favorites.tokens,
            favorite_devices: loaded_favorites.devices,
        }
    }
}

impl SendPageState {
    pub fn remove_favorite_device(&mut self, token: &str) {
        self.favorite_tokens.remove(token);
        self.favorite_devices.remove(token);
        self.persist_favorites();
    }

    pub fn add_or_update_favorite_device(
        &mut self,
        token: String,
        alias: String,
        ip: String,
        port: u16,
        https: bool,
        custom_alias: bool,
    ) {
        let ip = ip.trim().to_string();
        if ip.is_empty() || port == 0 {
            return;
        }
        let normalized_token = if token.trim().is_empty() {
            format!("{}:{}", ip, port)
        } else {
            token.trim().to_string()
        };
        self.favorite_tokens.insert(normalized_token.clone());
        self.favorite_devices.insert(
            normalized_token.clone(),
            FavoriteDevice {
                token: normalized_token,
                alias: alias.trim().to_string(),
                ip,
                port,
                https,
                custom_alias,
            },
        );
        self.persist_favorites();
    }

    pub fn sync_favorite_from_discovered(
        &mut self,
        token: &str,
        discovered_alias: &str,
        discovered_ip: &str,
        discovered_port: u16,
        discovered_https: bool,
    ) -> bool {
        if let Some(item) = self.favorite_devices.get_mut(token) {
            let old_alias = item.alias.clone();
            let old_ip = item.ip.clone();
            let old_port = item.port;
            let old_https = item.https;
            item.ip = discovered_ip.to_string();
            item.port = discovered_port;
            item.https = discovered_https;
            if !item.custom_alias {
                item.alias = discovered_alias.to_string();
            }
            return old_alias != item.alias
                || old_ip != item.ip
                || old_port != item.port
                || old_https != item.https;
        }
        false
    }

    pub fn persist_favorites_if_dirty(&self) {
        self.persist_favorites();
    }

    pub fn favorite_list_sorted(&self) -> Vec<FavoriteDevice> {
        let mut items: Vec<FavoriteDevice> = self.favorite_devices.values().cloned().collect();
        items.sort_by(|a, b| a.alias.cmp(&b.alias).then_with(|| a.ip.cmp(&b.ip)));
        items
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
        let mut devices: Vec<FavoriteDevice> = self.favorite_devices.values().cloned().collect();
        devices.sort_by(|a, b| a.alias.cmp(&b.alias).then_with(|| a.ip.cmp(&b.ip)));
        let payload = FavoritesData {
            favorite_tokens: tokens,
            favorite_devices: devices,
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
