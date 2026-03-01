//! Settings tab state and types (theme, color, receive/send/network options).
use serde::{Deserialize, Serialize};

/// Theme mode (Brightness)
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

/// Color mode
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ColorMode {
    #[default]
    System,
    LocalSend,
    Oled,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SendModeSetting {
    #[default]
    Single,
    Multiple,
    Link,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NetworkFilterMode {
    #[default]
    All,
    Whitelist,
    Blacklist,
}

/// Settings tab state
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct SettingsPageState {
    pub theme_mode: ThemeMode,
    pub color_mode: ColorMode,
    pub language: String,
    pub animations: bool,
    pub advanced: bool,
    pub quick_save: bool,
    pub quick_save_favorites: bool,
    pub require_pin: bool,
    pub receive_pin: String,
    pub destination: Option<String>,
    pub save_to_gallery: bool,
    pub auto_finish: bool,
    pub save_to_history: bool,
    pub send_mode_default: SendModeSetting,
    pub share_via_link_auto_accept: bool,
    #[serde(skip)]
    pub server_running: bool,
    #[serde(skip)]
    pub server_paused: bool,
    pub server_alias: String,
    pub server_port: u16,
    pub device_type: String,
    pub device_model: String,
    pub network_filtered: bool,
    pub network_filter_mode: NetworkFilterMode,
    pub network_filters: Vec<String>,
    pub discovery_target_subnets: Vec<String>,
    pub discovery_timeout: u32,
    pub encryption: bool,
    pub multicast_group: String,
}

impl SettingsPageState {
    pub fn load_or_default() -> Self {
        let path = crate::platform::preferences_path::get_preferences_file_path("settings.json");
        let Ok(raw) = std::fs::read_to_string(&path) else {
            return Self::default();
        };

        match serde_json::from_str::<Self>(&raw) {
            Ok(state) => state,
            Err(err) => {
                log::warn!("failed to parse settings file {}: {}", path.display(), err);
                Self::default()
            }
        }
    }

    pub fn persist_to_disk(&self) {
        let path = crate::platform::preferences_path::get_preferences_file_path("settings.json");
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
            log::warn!("failed to serialize settings state");
            return;
        };
        if let Err(err) = std::fs::write(&path, serialized) {
            log::warn!("failed to write settings file {}: {}", path.display(), err);
        }
    }
}

impl Default for SettingsPageState {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::System,
            color_mode: ColorMode::System,
            language: "System".to_string(),
            animations: true,
            advanced: false,
            quick_save: false,
            quick_save_favorites: false,
            require_pin: false,
            receive_pin: "123456".to_string(),
            destination: None,
            save_to_gallery: true,
            auto_finish: true,
            save_to_history: true,
            send_mode_default: SendModeSetting::Single,
            share_via_link_auto_accept: false,
            server_running: false,
            server_paused: false,
            server_alias: "NearSend".to_string(),
            server_port: 53317,
            device_type: "Desktop".to_string(),
            device_model: "".to_string(),
            network_filtered: false,
            network_filter_mode: NetworkFilterMode::All,
            network_filters: Vec::new(),
            discovery_target_subnets: Vec::new(),
            discovery_timeout: 900,
            encryption: false,
            multicast_group: "224.0.0.167".to_string(),
        }
    }
}
