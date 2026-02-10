//! Receive tab state and types (Quick Save, server info, incoming transfer, etc.).

use localsend::http::state::ClientInfo;
use localsend::model::transfer::FileDto;

/// Quick Save mode
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QuickSaveMode {
    Off,
    Favorites,
    On,
}

/// Incoming transfer request from another device.
#[derive(Clone)]
pub struct IncomingTransferRequest {
    pub sender: ClientInfo,
    pub files: Vec<FileDto>,
    pub session_id: String,
    pub selected_files: Vec<bool>,
}

/// Receive tab state
pub struct ReceivePageState {
    pub quick_save_mode: QuickSaveMode,
    pub show_advanced: bool,
    pub server_alias: String,
    pub server_ips: Vec<String>,
    pub server_port: u16,
    pub server_running: bool,
    pub incoming_request: Option<IncomingTransferRequest>,
    pub show_receive_dialog: bool,
}

impl Default for ReceivePageState {
    fn default() -> Self {
        Self {
            quick_save_mode: QuickSaveMode::Off,
            show_advanced: false,
            server_alias: "NearSend".to_string(),
            server_ips: vec!["192.168.1.100".to_string()],
            server_port: 53317,
            server_running: false,
            incoming_request: None,
            show_receive_dialog: false,
        }
    }
}
