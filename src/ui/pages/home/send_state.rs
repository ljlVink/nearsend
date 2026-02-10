//! Send tab state and types (selected files, nearby devices, send mode, etc.).

use localsend::http::state::ClientInfo;
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

/// Information about a selected file.
#[derive(Clone, Debug)]
pub struct SelectedFileInfo {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub file_type: String, // e.g. "image/png", "application/pdf", or extension
    pub text_content: Option<String>, // For text messages
}

/// Send tab state
pub struct SendPageState {
    pub selected_files: Vec<SelectedFileInfo>,
    pub selected_files_total_size: u64,
    pub send_content_type: SendContentType,
    pub nearby_devices: Vec<ClientInfo>,
    pub scanning: bool,
    pub local_ips: Vec<String>,
    pub send_mode: SendMode,
    pub help_index: usize,
    pub show_scan_menu: bool,
    pub show_send_mode_menu: bool,
    pub target_device: Option<ClientInfo>,
    pub target_ip: Option<String>,
    pub pending_send: bool,
}

impl Default for SendPageState {
    fn default() -> Self {
        Self {
            selected_files: Vec::new(),
            selected_files_total_size: 0,
            send_content_type: SendContentType::default(),
            nearby_devices: Vec::new(),
            scanning: false,
            local_ips: vec!["192.168.1.100".to_string()],
            send_mode: SendMode::Single,
            help_index: 0,
            show_scan_menu: false,
            show_send_mode_menu: false,
            target_device: None,
            target_ip: None,
            pending_send: false,
        }
    }
}
