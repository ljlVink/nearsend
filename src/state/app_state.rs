use crate::core::{discovery::DiscoveryService, server::ServerManager, transfer::TransferService};
use gpui::Entity;
use std::path::PathBuf;

/// Global application state
#[derive(Clone)]
pub struct AppState {
    pub discovery: Entity<DiscoveryService>,
    pub server: Entity<ServerManager>,
    pub transfer: Entity<TransferService>,
    pub selected_files: Vec<PathBuf>,
}

impl AppState {
    pub fn new(
        discovery: Entity<DiscoveryService>,
        server: Entity<ServerManager>,
        transfer: Entity<TransferService>,
    ) -> Self {
        Self {
            discovery,
            server,
            transfer,
            selected_files: Vec::new(),
        }
    }
}
