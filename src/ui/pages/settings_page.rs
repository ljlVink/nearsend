// Settings page state and types - aligned with localsend

/// Theme mode (Brightness)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

/// Color mode
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    System,
    LocalSend,
    Oled,
}

/// Settings page state
pub struct SettingsPageState {
    // General
    pub theme_mode: ThemeMode,
    pub color_mode: ColorMode,
    pub language: String,
    pub animations: bool,
    // Receive
    pub advanced: bool,
    pub quick_save: bool,
    pub quick_save_favorites: bool,
    pub require_pin: bool,
    pub destination: Option<String>,
    pub save_to_gallery: bool,
    pub auto_finish: bool,
    pub save_to_history: bool,
    // Send (advanced)
    pub share_via_link_auto_accept: bool,
    // Network
    pub server_running: bool,
    pub server_alias: String,
    pub server_port: u16,
    pub device_type: String,
    pub device_model: String,
    pub network_filtered: bool,
    pub discovery_timeout: u32,
    pub encryption: bool,
    pub multicast_group: String,
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
            destination: None,
            save_to_gallery: true,
            auto_finish: true,
            save_to_history: true,
            share_via_link_auto_accept: false,
            server_running: false,
            server_alias: "NearSend".to_string(),
            server_port: 53317,
            device_type: "Desktop".to_string(),
            device_model: "".to_string(),
            network_filtered: false,
            discovery_timeout: 900,
            encryption: false,
            multicast_group: "239.255.255.250".to_string(),
        }
    }
}

// Page rendering is implemented in app.rs
