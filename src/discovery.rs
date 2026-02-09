use crate::protocol::{DeviceInfo, MULTICAST_ADDRESS, MULTICAST_PORT, PROTOCOL_VERSION};
use anyhow::{Context, Result};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

/// Discovery service for finding nearby devices
#[derive(Clone)]
pub struct DiscoveryService {
    socket: Arc<UdpSocket>,
    device_info: DeviceInfo,
    device_tx: mpsc::UnboundedSender<DeviceInfo>,
}

impl DiscoveryService {
    /// Create a new discovery service
    pub fn new(
        device_info: DeviceInfo,
        device_tx: mpsc::UnboundedSender<DeviceInfo>,
    ) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .context("Failed to bind UDP socket for discovery")?;
        socket.set_multicast_loop_v4(true)?;
        socket.set_broadcast(true)?;

        Ok(Self {
            socket: Arc::new(socket),
            device_info,
            device_tx,
        })
    }

    /// Start the discovery service
    pub async fn start(&self) -> Result<()> {
        let socket = Arc::clone(&self.socket);
        let device_info = self.device_info.clone();
        let device_tx = self.device_tx.clone();

        // Start broadcasting device info
        let socket_clone = Arc::clone(&socket);
        let device_info_clone = device_info.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(2));
            let multicast_addr: SocketAddr = format!("{}:{}", MULTICAST_ADDRESS, MULTICAST_PORT)
                .parse()
                .expect("Invalid multicast address");

            loop {
                interval.tick().await;
                let payload = match serde_json::to_vec(&device_info_clone) {
                    Ok(p) => p,
                    Err(e) => {
                        log::warn!("Failed to serialize device info: {}", e);
                        continue;
                    }
                };
                if let Err(e) = socket_clone.send_to(&payload, multicast_addr) {
                    log::warn!("Failed to broadcast announcement: {}", e);
                }
            }
        });

        // Start listening for other devices
        let socket = Arc::clone(&self.socket);
        let device_tx = self.device_tx.clone();
        let our_fingerprint = self.device_info.fingerprint.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                        match socket.recv_from(&mut buf) {
                    Ok((size, addr)) => {
                        if let Ok(mut announcement) = serde_json::from_slice::<DeviceInfo>(&buf[..size])
                        {
                            // Ignore our own announcements
                            if announcement.fingerprint != our_fingerprint {
                                // Store the IP address from the packet
                                announcement.ip_address = Some(addr.ip().to_string());
                                if let Err(e) = device_tx.send(announcement) {
                                    log::error!("Failed to send device info: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to receive discovery packet: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

}

/// Create default device info
pub fn create_device_info(alias: String, port: u16, use_https: bool) -> DeviceInfo {
    use crate::protocol::DeviceType;
    use uuid::Uuid;

    DeviceInfo {
        alias,
        version: PROTOCOL_VERSION.to_string(),
        device_model: get_device_model(),
        device_type: DeviceType::Desktop,
        fingerprint: Uuid::new_v4().to_string(),
        port,
        protocol: if use_https { "https" } else { "http" }.to_string(),
        download: true,
        ip_address: None,
    }
}

fn get_device_model() -> String {
    #[cfg(target_os = "macos")]
    return "macOS".to_string();
    #[cfg(target_os = "windows")]
    return "Windows".to_string();
    #[cfg(target_os = "linux")]
    return "Linux".to_string();
    #[cfg(target_arch = "aarch64")]
    return "ARM64".to_string();
    "Unknown".to_string()
}
