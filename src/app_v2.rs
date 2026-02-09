// New implementation using localsend-rs crate
use gpui::{
    div, prelude::*, px, Context, Entity, SharedString, Window,
};
use gpui_component::{
    button::Button,
    v_flex, ActiveTheme as _, Root, StyledExt as _,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main application state using localsend-rs
pub struct NearSendAppV2 {
    // Device discovery - will use localsend-rs
    devices: Entity<DeviceListState>,
    
    // File selection
    selected_files: Vec<PathBuf>,
    
    // Transfers
    transfers: Entity<TransferListState>,
    
    _subscriptions: Vec<gpui::Subscription>,
}

/// Device list state
#[derive(Clone)]
pub struct DeviceListState {
    devices: Arc<RwLock<HashMap<String, localsend_rs::Device>>>,
}

impl DeviceListState {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_device(&self, device: localsend_rs::Device) {
        // Use fingerprint or another unique identifier as key
        let key = device.fingerprint.clone();
        self.devices.write().await.insert(key, device);
    }

    pub async fn get_devices(&self) -> Vec<localsend_rs::Device> {
        self.devices.read().await.values().cloned().collect()
    }
}

/// Transfer list state
#[derive(Clone)]
pub struct TransferListState {
    transfers: Arc<RwLock<HashMap<String, TransferInfo>>>,
}

#[derive(Clone)]
pub struct TransferInfo {
    pub id: String,
    pub status: String,
    pub progress: f64,
    pub file_name: String,
}

impl TransferListState {
    pub fn new() -> Self {
        Self {
            transfers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_transfer(&self, transfer: TransferInfo) {
        self.transfers.write().await.insert(transfer.id.clone(), transfer);
    }

    pub async fn get_transfers(&self) -> Vec<TransferInfo> {
        self.transfers.read().await.values().cloned().collect()
    }
}

impl NearSendAppV2 {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> anyhow::Result<Self> {
        let devices = cx.new(|_| DeviceListState::new());
        let transfers = cx.new(|_| TransferListState::new());

        // TODO: Initialize localsend-rs discovery and server
        // This will use the localsend-rs crate APIs

        Ok(Self {
            devices,
            selected_files: Vec::new(),
            transfers,
            _subscriptions: Vec::new(),
        })
    }
}

impl gpui::Render for NearSendAppV2 {
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
                    .child("NearSend - Using localsend-rs"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Protocol implementation provided by localsend-rs crate"),
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
                            .child("Device discovery using localsend-rs..."),
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
                            .child("Transfer list..."),
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
