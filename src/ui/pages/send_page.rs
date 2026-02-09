// Send page state and types
use localsend::http::state::ClientInfo;

/// Send page state
pub struct SendPageState {
    pub selected_files: Vec<std::path::PathBuf>,
    pub selected_files_total_size: u64, // Total size in bytes
    pub nearby_devices: Vec<ClientInfo>,
    pub scanning: bool,
    pub local_ips: Vec<String>, // For scan button IP selection
    pub send_mode: SendMode,
    pub help_index: usize, // For OpacitySlideshow
    pub show_scan_menu: bool,
    pub show_send_mode_menu: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SendMode {
    Single,
    Multiple,
    Link,
}

impl Default for SendPageState {
    fn default() -> Self {
        Self {
            selected_files: Vec::new(),
            selected_files_total_size: 0,
            nearby_devices: Vec::new(),
            scanning: false,
            local_ips: vec!["192.168.1.100".to_string()],
            send_mode: SendMode::Single,
            help_index: 0,
            show_scan_menu: false,
            show_send_mode_menu: false,
        }
    }
}

// Page rendering is implemented in app.rs
