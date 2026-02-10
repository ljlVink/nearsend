use crate::core::{
    cert::CertPair, discovery::DiscoveryService, server::ServerManager, transfer::TransferService,
};
use gpui::Entity;
use localsend::http::state::ClientInfo;
use std::path::PathBuf;
use tokio::runtime::Handle;

/// Global application state
#[derive(Clone)]
pub struct AppState {
    pub discovery: Entity<DiscoveryService>,
    pub server: Entity<ServerManager>,
    pub transfer: Entity<TransferService>,
    pub selected_files: Vec<PathBuf>,
    pub client_info: Option<ClientInfo>,
    pub cert: Option<CertPair>,
    pub tokio_handle: Handle,
}

impl AppState {
    pub fn new(
        discovery: Entity<DiscoveryService>,
        server: Entity<ServerManager>,
        transfer: Entity<TransferService>,
        tokio_handle: Handle,
    ) -> Self {
        Self {
            discovery,
            server,
            transfer,
            selected_files: Vec::new(),
            client_info: None,
            cert: None,
            tokio_handle,
        }
    }
}
