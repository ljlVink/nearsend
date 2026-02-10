use crate::core::{cert::CertPair, server::ServerManager};
use gpui::Entity;
use localsend::http::state::ClientInfo;
use std::path::PathBuf;
use tokio::runtime::Handle;

/// Global application state
#[derive(Clone)]
pub struct AppState {
    pub server: Entity<ServerManager>,
    pub selected_files: Vec<PathBuf>,
    pub client_info: Option<ClientInfo>,
    pub cert: Option<CertPair>,
    pub tokio_handle: Handle,
}

impl AppState {
    pub fn new(
        server: Entity<ServerManager>,
        tokio_handle: Handle,
    ) -> Self {
        Self {
            server,
            selected_files: Vec::new(),
            client_info: None,
            cert: None,
            tokio_handle,
        }
    }
}
