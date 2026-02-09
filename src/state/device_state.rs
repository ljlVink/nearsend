use localsend::http::state::ClientInfo;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Device state management
#[derive(Clone)]
pub struct DeviceState {
    devices: Arc<RwLock<HashMap<String, ClientInfo>>>,
}

impl DeviceState {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_device(&self, device: ClientInfo) {
        self.devices.write().await.insert(device.token.clone(), device);
    }

    pub async fn remove_device(&self, token: &str) {
        self.devices.write().await.remove(token);
    }

    pub async fn get_devices(&self) -> Vec<ClientInfo> {
        self.devices.read().await.values().cloned().collect()
    }

    pub async fn get_device(&self, token: &str) -> Option<ClientInfo> {
        self.devices.read().await.get(token).cloned()
    }
}

impl Default for DeviceState {
    fn default() -> Self {
        Self::new()
    }
}
