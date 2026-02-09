use crate::client::LocalSendClient;
use crate::discovery::{create_device_info, DiscoveryService};
use crate::protocol::{DeviceInfo, FileInfo, TransferProgress, TransferStatus};
use crate::server::{create_router, ServerState};
use anyhow::Result;
use axum::Server;
use gpui::{
    div, prelude::*, px, Context, Entity, SharedString, Window,
};
use gpui_component::{
    button::Button,
    v_flex, ActiveTheme as _, Root, StyledExt as _,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Main application state
pub struct NearSendApp {
    // Device discovery
    devices: Entity<DeviceListState>,
    discovery_service: Option<DiscoveryService>,
    
    // File selection
    selected_files: Vec<PathBuf>,
    
    // Transfers
    transfers: Entity<TransferListState>,
    
    // Server
    server_port: u16,
    device_info: DeviceInfo,
    
    _subscriptions: Vec<gpui::Subscription>,
}

/// Device list state
#[derive(Clone)]
pub struct DeviceListState {
    devices: Arc<RwLock<HashMap<String, DeviceInfo>>>,
}

impl DeviceListState {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn add_device(&self, device: DeviceInfo) {
        self.devices.write().await.insert(device.fingerprint.clone(), device);
    }

    async fn get_devices(&self) -> Vec<DeviceInfo> {
        self.devices.read().await.values().cloned().collect()
    }

    async fn remove_device(&self, fingerprint: &str) {
        self.devices.write().await.remove(fingerprint);
    }
}

/// Transfer list state
#[derive(Clone)]
#[derive(Clone)]
pub struct TransferListState {
    transfers: Arc<RwLock<HashMap<String, TransferProgress>>>,
}

impl TransferListState {
    pub fn new() -> Self {
        Self {
            transfers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn add_transfer(&self, transfer: TransferProgress) {
        self.transfers.write().await.insert(transfer.transfer_id.clone(), transfer);
    }

    async fn get_transfers(&self) -> Vec<TransferProgress> {
        self.transfers.read().await.values().cloned().collect()
    }

    async fn update_transfer(&self, transfer_id: &str, status: TransferStatus, bytes_sent: u64) {
        if let Some(transfer) = self.transfers.write().await.get_mut(transfer_id) {
            transfer.status = status;
            transfer.bytes_sent = bytes_sent;
        }
    }
}

impl NearSendApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Result<Self> {
        let server_port = 53317;
        let device_info = create_device_info("NearSend".to_string(), server_port, false);
        
        let devices = cx.new(|_| DeviceListState::new());
        let transfers = cx.new(|_| TransferListState::new());

        // Create discovery service
        let (device_tx, mut device_rx) = mpsc::unbounded_channel();
        let discovery_service = DiscoveryService::new(device_info.clone(), device_tx.clone())?;

        // Start discovery service
        let discovery_service_clone = discovery_service.clone();
        tokio::spawn(async move {
            if let Err(e) = discovery_service_clone.start().await {
                log::error!("Failed to start discovery service: {}", e);
            }
        });

        // Handle discovered devices
        let devices_clone2 = devices.clone();
        tokio::spawn(async move {
            while let Some(device) = device_rx.recv().await {
                devices_clone2.add_device(device).await;
            }
        });

        // Start HTTP server
        let (transfer_tx, mut transfer_rx) = mpsc::unbounded_channel();
        let server_state = ServerState {
            transfers: Arc::new(RwLock::new(HashMap::new())),
            on_transfer_request: Arc::new(transfer_tx),
        };

        let transfers_clone = transfers.clone();
        tokio::spawn(async move {
            let app = create_router(server_state);
            let addr = SocketAddr::from(([0, 0, 0, 0], server_port));
            let server = Server::bind(&addr).serve(app.into_make_service());
            
            log::info!("Server started on {}", addr);
            
            if let Err(e) = server.await {
                log::error!("Server error: {}", e);
            }
        });

        // Handle incoming transfer requests
        let transfers_clone2 = transfers.clone();
        tokio::spawn(async move {
            while let Some((transfer_id, request)) = transfer_rx.recv().await {
                // Create transfer progress
                let total_bytes: u64 = request.files.iter().map(|f| f.file_size).sum();
                let transfer = TransferProgress {
                    transfer_id: transfer_id.clone(),
                    status: TransferStatus::Pending,
                    bytes_sent: 0,
                    total_bytes,
                    file_index: 0,
                    file_name: request.files.first().map(|f| f.file_name.clone()).unwrap_or_default(),
                };
                transfers_clone2.add_transfer(transfer).await;
            }
        });

        Ok(Self {
            devices,
            discovery_service: Some(discovery_service),
            selected_files: Vec::new(),
            transfers,
            server_port,
            device_info,
            _subscriptions: Vec::new(),
        })
    }

    pub async fn send_files(&self, device: &DeviceInfo, files: Vec<PathBuf>) -> Result<()> {
        let client = LocalSendClient::new(self.device_info.clone())?;
        
        let file_infos: Vec<FileInfo> = files
            .iter()
            .enumerate()
            .map(|(idx, path)| {
                let file_size = std::fs::metadata(path)
                    .map(|m| m.len())
                    .unwrap_or(0);
                FileInfo {
                    id: format!("file_{}", idx),
                    file_name: path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    file_size,
                    file_type: None,
                }
            })
            .collect();

        let request = crate::protocol::TransferRequest {
            files: file_infos.clone(),
            text: None,
        };

        let response = client.send_transfer(device, request).await?;
        
        // TODO: Upload files
        log::info!("Transfer initiated: {}", response.transfer_id);
        
        Ok(())
    }

    fn render_device_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let devices_state = self.devices.clone();
        let transfers_state = self.transfers.clone();
        let selected_files = self.selected_files.clone();
        
        // Use a placeholder for now - in a real implementation, we'd poll the state
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Scanning for devices on local network...")
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Make sure devices are on the same network and firewall allows port 53317")
            )
    }

    fn render_transfer_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("No active transfers")
            )
    }
}

impl Render for NearSendApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        let content = v_flex()
            .size_full()
            .p_4()
            .gap_4()
            .bg(cx.theme().background)
            .child(
                div()
                    .text_xl()
                    .font_bold()
                    .text_color(cx.theme().foreground)
                    .child("NearSend - LocalSend Compatible Client"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("Device: {} | Port: {}", self.device_info.alias, self.server_port)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("Nearby Devices"),
                    )
                    .child(
                        div()
                            .bg(cx.theme().muted)
                            .rounded_lg()
                            .p_4()
                            .min_h(px(200.))
                            .child(self.render_device_list(cx)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("File Selection"),
                    )
                            .child(
                                Button::new("Select Files")
                                    .on_click(cx.listener(|_this, _event, _window, _cx| {
                                        // TODO: Implement file picker
                                        log::info!("File selection clicked");
                                    })),
                            ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("Transfers"),
                    )
                    .child(
                        div()
                            .bg(cx.theme().muted)
                            .rounded_lg()
                            .p_4()
                            .min_h(px(150.))
                            .child(self.render_transfer_list(cx)),
                    ),
            );

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(content)
            .children(sheet_layer)
            .children(dialog_layer)
            .children(notification_layer)
    }
}
