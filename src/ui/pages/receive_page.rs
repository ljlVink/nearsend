// Receive page state and types

/// Quick Save mode
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QuickSaveMode {
    Off,
    Favorites,
    On,
}

/// Receive page state
pub struct ReceivePageState {
    pub quick_save_mode: QuickSaveMode,
    pub show_advanced: bool,
    pub show_history_button: bool,
    pub server_alias: String,
    pub server_ips: Vec<String>, // Support multiple IPs like localsend
    pub server_port: u16,
    pub server_running: bool,
}

impl Default for ReceivePageState {
    fn default() -> Self {
        Self {
            quick_save_mode: QuickSaveMode::Off,
            show_advanced: false,
            show_history_button: false,
            server_alias: "NearSend".to_string(),
            server_ips: vec!["192.168.1.100".to_string()],
            server_port: 53317,
            server_running: false,
        }
    }
}

// Page rendering is implemented in app.rs
