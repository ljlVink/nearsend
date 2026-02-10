use localsend::http::state::ClientInfo;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Device discovery service using LocalSend core
pub struct DiscoveryService {
    devices: Arc<RwLock<HashMap<String, ClientInfo>>>,
}

impl DiscoveryService {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start device discovery (sync wrapper).
    pub fn start_sync(&self) {
        log::info!("Starting device discovery service");
        // TODO: Initialize multicast discovery using localsend core
        // This will use the official discovery mechanism when implemented
    }

    /// Get discovered devices
    pub async fn get_devices(&self) -> Vec<ClientInfo> {
        self.devices.read().await.values().cloned().collect()
    }

    /// Stop discovery service
    pub fn stop(&self) {
        log::info!("Stopping device discovery service");
    }
}

impl Default for DiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}
