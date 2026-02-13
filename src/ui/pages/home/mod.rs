//! Home page: three tabs (Receive, Send, Settings) with bottom navigation.
//! Uses gpui-router; history is a separate route (see history page).

mod receive_state;
mod receive_tab;
mod send_state;
mod send_tab;
mod settings_state;
mod settings_tab;

pub use receive_state::{IncomingTransferRequest, QuickSaveMode, ReceivePageState};
pub use send_state::{
    SelectedFileInfo, SendContentType, SendMode, SendPageState, SendSessionStatus,
};
pub use settings_state::{
    ColorMode, NetworkFilterMode, SendModeSetting, SettingsPageState, ThemeMode,
};

use crate::state::{
    app_state::AppState,
    device_state::DeviceState,
    history_state::{HistoryEntry, HistoryState},
    receive_inbox_state::ReceiveInboxState,
    send_selection_state::SendSelectionState,
    transfer_state::{TransferDirection, TransferState, TransferStatus},
};
use gpui::{div, prelude::*, px, AnyElement, Context, Entity, IntoElement, Window};
use gpui_component::button::{Button, ButtonCustomVariant, ButtonVariants as _};
use gpui_component::input::{Input, InputState};
use gpui_component::select::{Select, SelectEvent, SelectState};
use gpui_component::{
    h_flex, v_flex, ActiveTheme as _, Icon, IndexPath, Sizable as _, Size, StyledExt as _,
    WindowExt as _,
};
use gpui_router::RouterState;
use localsend::http::state::ClientInfo;
use localsend::model::discovery::DeviceType;
use std::collections::BTreeSet;
use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;

/// Tab identifier for home page bottom navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabType {
    Receive,
    Send,
    Settings,
}

/// Home page: receives / send / settings tabs + bottom nav.
pub struct HomePage {
    pub(super) app_state: Entity<AppState>,
    pub(super) device_state: Entity<DeviceState>,
    pub(super) transfer_state: Entity<TransferState>,
    pub(super) history_state: Entity<HistoryState>,
    pub(super) send_selection_state: Entity<SendSelectionState>,
    pub(super) receive_inbox_state: Entity<ReceiveInboxState>,
    pub(super) current_tab: TabType,
    services_started: bool,
    pub(super) receive_state: ReceivePageState,
    pub(super) send_state: SendPageState,
    pub(super) settings_state: SettingsPageState,
    // Select states for settings dropdowns (lazy-initialized on first render)
    pub(super) theme_select: Option<Entity<SelectState<Vec<&'static str>>>>,
    pub(super) color_select: Option<Entity<SelectState<Vec<&'static str>>>>,
    pub(super) language_select: Option<Entity<SelectState<Vec<&'static str>>>>,
    // Text input state for the message input dialog
    pub(super) text_input_state: Option<Entity<InputState>>,
    // Input states for the send-to-address dialog
    pub(super) send_ip_input_state: Option<Entity<InputState>>,
}

impl HomePage {
    fn normalized_peer_token(info: &ClientInfo, ip: &str, port: u16) -> String {
        if info.token.is_empty() {
            format!("{}:{}", ip, port)
        } else {
            info.token.clone()
        }
    }

    fn normalize_peer_info(mut info: ClientInfo, ip: &str, port: u16) -> ClientInfo {
        if info.token.is_empty() {
            info.token = format!("{}:{}", ip, port);
        }
        info
    }

    pub fn new(
        app_state: Entity<AppState>,
        device_state: Entity<DeviceState>,
        transfer_state: Entity<TransferState>,
        history_state: Entity<HistoryState>,
        send_selection_state: Entity<SendSelectionState>,
        receive_inbox_state: Entity<ReceiveInboxState>,
    ) -> Self {
        let alias = generate_random_alias();
        let mut receive_state = ReceivePageState::default();
        let settings_file_exists =
            crate::platform::preferences_path::get_preferences_file_path("settings.json").exists();
        let mut settings_state = SettingsPageState::load_or_default();
        if !settings_file_exists || settings_state.server_alias.trim().is_empty() {
            settings_state.server_alias = alias;
            settings_state.persist_to_disk();
        }
        receive_state.server_alias = settings_state.server_alias.clone();
        receive_state.quick_save_mode = if settings_state.quick_save {
            QuickSaveMode::On
        } else if settings_state.quick_save_favorites {
            QuickSaveMode::Favorites
        } else {
            QuickSaveMode::Off
        };

        let mut send_state = SendPageState::default();
        send_state.send_mode = match settings_state.send_mode_default {
            SendModeSetting::Single => SendMode::Single,
            SendModeSetting::Multiple => SendMode::Multiple,
            SendModeSetting::Link => SendMode::Link,
        };

        Self {
            app_state,
            device_state,
            transfer_state,
            history_state,
            send_selection_state,
            receive_inbox_state,
            current_tab: TabType::Receive,
            services_started: false,
            receive_state,
            send_state,
            settings_state,
            theme_select: None,
            color_select: None,
            language_select: None,
            text_input_state: None,
            send_ip_input_state: None,
        }
    }

    pub(super) fn persist_settings(&self) {
        self.settings_state.persist_to_disk();
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AddressInputMode {
    Label,
    IpAddress,
}

enum ClipboardPickOutcome {
    Success(String),
    PermissionDenied,
    ReadFailed,
}

enum PathPickOutcome {
    Success(Vec<std::path::PathBuf>),
    Cancelled,
    Failed,
}

enum SendUiMessage {
    Notice(String),
    UpdateStatus {
        status: SendSessionStatus,
        message: Option<String>,
    },
    RequestPin {
        show_invalid_pin: bool,
        responder: oneshot::Sender<Option<String>>,
    },
}

impl gpui::Render for HomePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.poll_incoming_events(window, cx);
        self.sync_selected_files_from_shared(cx);
        if !self.services_started {
            self.services_started = true;
            // Initialize select states for settings dropdowns
            self.init_select_states(window, cx);
            // Start server and discovery services
            self.start_services(cx);
        }

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .child(match self.current_tab {
                        TabType::Receive => receive_tab::render_receive_content(self, window, cx),
                        TabType::Send => send_tab::render_send_content(self, window, cx),
                        TabType::Settings => {
                            settings_tab::render_settings_content(self, window, cx)
                        }
                    }),
            )
            .child(
                div()
                    .w_full()
                    .bg(cx.theme().background)
                    .py(px(6.))
                    .child(self.render_bottom_nav(cx)),
            )
    }
}

impl HomePage {
    pub(super) fn send_mode_label(mode: SendMode) -> &'static str {
        match mode {
            SendMode::Single => "单设备",
            SendMode::Multiple => "多设备",
            SendMode::Link => "链接分享",
        }
    }

    pub(super) fn apply_send_mode_default(&mut self, mode: SendMode) {
        self.send_state.send_mode = mode;
        self.settings_state.send_mode_default = match mode {
            SendMode::Single => SendModeSetting::Single,
            SendMode::Multiple => SendModeSetting::Multiple,
            SendMode::Link => SendModeSetting::Link,
        };
        self.persist_settings();
    }

    pub(crate) fn poll_incoming_events(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let events = crate::core::receive_events::drain_incoming_events();
        if events.is_empty() {
            return;
        }
        log::info!("poll_incoming_events: received {} events", events.len());

        let mut should_open_receive_page = false;
        let mut should_auto_finish_receive = false;
        let mut history_entries = Vec::new();
        let settings_quick_save = self.settings_state.quick_save;
        let settings_quick_save_favorites = self.settings_state.quick_save_favorites;
        let save_to_history = self.settings_state.save_to_history;
        let auto_finish = self.settings_state.auto_finish;
        let favorite_tokens = self.send_state.favorite_tokens.clone();

        self.receive_inbox_state.update(cx, |state, _| {
            for event in events {
                match &event {
                    crate::core::receive_events::IncomingTransferEvent::Prepared {
                        session_id,
                        sender_fingerprint,
                        files,
                        ..
                    } => {
                        let is_message_only = files.len() == 1
                            && files
                                .first()
                                .map(|f| f.file_type.starts_with("text/") && f.preview.is_some())
                                .unwrap_or(false);
                        let is_favorite = favorite_tokens.contains(sender_fingerprint);
                        if is_message_only {
                            crate::core::receive_events::submit_incoming_decision(
                                session_id.clone(),
                                crate::core::receive_events::IncomingTransferDecision::AcceptAll,
                            );
                            should_open_receive_page = true;
                        }
                        let quick_save = !is_message_only
                            && (settings_quick_save
                                || (settings_quick_save_favorites && is_favorite));
                        if quick_save {
                            crate::core::receive_events::submit_incoming_decision(
                                session_id.clone(),
                                crate::core::receive_events::IncomingTransferDecision::AcceptAll,
                            );
                        } else {
                            should_open_receive_page = true;
                        }
                    }
                    crate::core::receive_events::IncomingTransferEvent::FileReceived { .. } => {
                        if let Some(active) = state.active.as_ref() {
                            let is_favorite = favorite_tokens.contains(&active.sender_fingerprint);
                            let quick_save = !active.is_message_only
                                && (settings_quick_save
                                    || (settings_quick_save_favorites && is_favorite));
                            if !quick_save {
                                should_open_receive_page = true;
                            }
                        }

                        if let crate::core::receive_events::IncomingTransferEvent::FileReceived {
                            session_id,
                            file_id,
                            saved_path,
                            ..
                        } = &event
                        {
                            if let Some(active) = state.active.as_ref() {
                                if active.session_id == *session_id {
                                    if let Some(item) =
                                        active.items.iter().find(|x| x.file_id == *file_id)
                                    {
                                        let file_path = saved_path
                                            .as_ref()
                                            .map(std::path::PathBuf::from)
                                            .unwrap_or_default();
                                        let timestamp = SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .map(|d| d.as_secs())
                                            .unwrap_or(0);
                                        if save_to_history {
                                            history_entries.push(HistoryEntry {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                file_name: item.file_name.clone(),
                                                file_size: item.size,
                                                file_path,
                                                direction: TransferDirection::Receive,
                                                device_name: active.sender_alias.clone(),
                                                timestamp,
                                                status: TransferStatus::Completed,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    crate::core::receive_events::IncomingTransferEvent::Completed { .. } => {
                        if auto_finish {
                            should_auto_finish_receive = true;
                        }
                    }
                    _ => {}
                }
                state.apply_event(event);
            }
        });

        if should_auto_finish_receive {
            self.receive_inbox_state
                .update(cx, |state, _| state.clear());
            if RouterState::global(cx).location.pathname == "/receive/incoming" {
                RouterState::global_mut(cx).location.pathname = "/".into();
                window.refresh();
            }
        }

        for entry in history_entries {
            let _ = self.history_state.update(cx, |state, _| {
                state.add_entry(entry);
            });
        }

        if should_open_receive_page {
            log::info!("poll_incoming_events: navigate to /receive/incoming");
            RouterState::global_mut(cx).location.pathname = "/receive/incoming".into();
            window.refresh();
        }
    }

    fn sync_selected_files_from_shared(&mut self, cx: &mut Context<Self>) {
        let items = self.send_selection_state.read(cx).items().to_vec();
        let total = self.send_selection_state.read(cx).total_size();
        self.send_state.selected_files = items
            .into_iter()
            .map(|item| send_state::SelectedFileInfo {
                path: item.path,
                name: item.name,
                size: item.size,
                file_type: item.file_type,
                text_content: item.text_content,
            })
            .collect();
        self.send_state.selected_files_total_size = total;
    }

    /// Start the HTTP server and discovery service.
    fn start_services(&mut self, cx: &mut Context<Self>) {
        let local_ips = detect_local_ipv4s(&self.settings_state);
        log::info!("near-send local ipv4 candidates: {:?}", local_ips);
        self.receive_state.server_ips = local_ips.clone();
        self.send_state.local_ips = local_ips;

        let alias = self.receive_state.server_alias.clone();
        let token = uuid::Uuid::new_v4().to_string();
        let port = self.receive_state.server_port;
        let use_https = self.settings_state.encryption;
        let device_model = normalized_device_model(&self.settings_state);
        let device_type = parse_device_type(&self.settings_state.device_type);
        let client_info = ClientInfo {
            alias: alias.clone(),
            version: "2.1".to_string(),
            device_model: device_model.clone(),
            device_type,
            token: token.clone(),
        };

        // Store client_info in AppState
        let app_state_entity = self.app_state.clone();
        app_state_entity.update(cx, |state, _cx| {
            state.client_info = Some(client_info.clone());
        });

        self.start_local_server(cx);

        let tokio_handle = self.app_state.read(cx).tokio_handle.clone();
        tokio_handle.spawn(async move {
            if let Err(err) = crate::core::multicast::start_multicast_service(
                client_info.alias,
                client_info.token,
                port,
                use_https,
                device_model,
                client_info.device_type,
            )
            .await
            {
                log::warn!("multicast service failed: {}", err);
            }
        });
    }

    pub(super) fn start_local_server(&mut self, cx: &mut Context<Self>) {
        self.sync_server_config_to_runtime(cx);
        let client_info = self
            .app_state
            .read(cx)
            .client_info
            .clone()
            .unwrap_or_else(|| ClientInfo {
                alias: self.settings_state.server_alias.clone(),
                version: "2.1".to_string(),
                device_model: normalized_device_model(&self.settings_state),
                device_type: parse_device_type(&self.settings_state.device_type),
                token: uuid::Uuid::new_v4().to_string(),
            });

        let server_entity = self.app_state.read(cx).server.clone();
        let tokio_handle = self.app_state.read(cx).tokio_handle.clone();
        let cert = self.app_state.read(cx).cert.clone();
        let use_https = self.settings_state.encryption;
        match server_entity.update(cx, |server, _cx| {
            server.set_port(self.settings_state.server_port);
            server.start(client_info, use_https, cert, &tokio_handle)
        }) {
            Ok(()) => {
                self.receive_state.server_running = true;
                self.settings_state.server_running = true;
                self.settings_state.server_paused = false;
            }
            Err(e) => {
                log::error!("Failed to start server: {}", e);
                self.receive_state.server_running = false;
                self.settings_state.server_running = false;
            }
        }
    }

    pub(super) fn stop_local_server(&mut self, cx: &mut Context<Self>) {
        let server_entity = self.app_state.read(cx).server.clone();
        server_entity.update(cx, |server, _cx| {
            server.stop();
        });
        self.receive_state.server_running = false;
        self.settings_state.server_running = false;
    }

    pub(super) fn sync_server_config_to_runtime(&mut self, cx: &mut Context<Self>) {
        let local_ips = detect_local_ipv4s(&self.settings_state);
        self.receive_state.server_ips = local_ips.clone();
        self.send_state.local_ips = local_ips;
        self.settings_state.network_filtered = is_network_filter_active(&self.settings_state);
        self.receive_state.server_alias = self.settings_state.server_alias.clone();
        self.receive_state.server_port = self.settings_state.server_port;
        let require_pin = self.settings_state.require_pin;
        let receive_pin = self.settings_state.receive_pin.clone();
        let server_entity = self.app_state.read(cx).server.clone();
        let tokio_handle = self.app_state.read(cx).tokio_handle.clone();
        let default_save_directory = self
            .settings_state
            .destination
            .as_ref()
            .map(std::path::PathBuf::from);
        server_entity.update(cx, |server, _| {
            server.set_receive_pin_config(require_pin, receive_pin, &tokio_handle);
            server.set_default_save_directory(default_save_directory, &tokio_handle);
        });

        let alias = self.settings_state.server_alias.clone();
        let device_model = normalized_device_model(&self.settings_state);
        let device_type = parse_device_type(&self.settings_state.device_type);
        let app_state_entity = self.app_state.clone();
        app_state_entity.update(cx, |state, _| {
            if let Some(info) = state.client_info.as_mut() {
                info.alias = alias.clone();
                info.device_model = device_model.clone();
                info.device_type = device_type;
            } else {
                state.client_info = Some(ClientInfo {
                    alias: alias.clone(),
                    version: "2.1".to_string(),
                    device_model,
                    device_type,
                    token: uuid::Uuid::new_v4().to_string(),
                });
            }
        });
    }

    pub(super) fn restart_local_server_with_current_config(&mut self, cx: &mut Context<Self>) {
        self.sync_server_config_to_runtime(cx);
        self.stop_local_server(cx);
        self.start_local_server(cx);
    }

    pub(super) fn pause_local_server(&mut self, cx: &mut Context<Self>) {
        self.stop_local_server(cx);
        self.settings_state.server_paused = true;
    }

    pub(super) fn resume_local_server(&mut self, cx: &mut Context<Self>) {
        self.sync_server_config_to_runtime(cx);
        self.start_local_server(cx);
    }

    pub(super) fn hydrate_nearby_devices_from_cache(&mut self, cx: &mut Context<Self>) {
        let own = self
            .app_state
            .read(cx)
            .client_info
            .as_ref()
            .map(|c| c.token.clone())
            .unwrap_or_default();
        let cached = crate::core::discovery::list_passive_devices(Some(&own));
        if cached.is_empty() {
            return;
        }

        let mut endpoint_map = self.send_state.nearby_endpoints.clone();
        for d in &cached {
            let key = Self::normalized_peer_token(&d.info, &d.ip, d.port);
            endpoint_map.insert(
                key,
                send_state::DeviceEndpoint {
                    ip: d.ip.clone(),
                    port: d.port,
                    https: d.https,
                },
            );
        }
        self.send_state.nearby_endpoints = endpoint_map;

        let mut info_map = std::collections::HashMap::<String, ClientInfo>::new();
        for info in self.send_state.nearby_devices.iter().cloned() {
            info_map.insert(info.token.clone(), info);
        }
        let mut favorites_changed = false;
        for d in cached {
            let normalized = Self::normalize_peer_info(d.info, &d.ip, d.port);
            favorites_changed |= self.send_state.sync_favorite_from_discovered(
                &normalized.token,
                &normalized.alias,
                &d.ip,
                d.port,
                d.https,
            );
            info_map.insert(normalized.token.clone(), normalized);
        }
        if favorites_changed {
            self.send_state.persist_favorites_if_dirty();
        }
        self.send_state.nearby_devices = info_map.into_values().collect();
        self.send_state
            .nearby_devices
            .sort_by(|a, b| a.alias.cmp(&b.alias));
    }

    pub(super) fn start_discovery_scan(&mut self, force_refresh: bool, cx: &mut Context<Self>) {
        if self.send_state.scanning {
            return;
        }
        self.send_state.has_scanned_once = true;
        self.send_state.scanning = true;
        if force_refresh {
            self.send_state.nearby_devices.clear();
            self.send_state.nearby_endpoints.clear();
            crate::core::discovery::clear_passive_devices();
        }

        let port = self.receive_state.server_port;
        let timeout_ms = self.settings_state.discovery_timeout.max(200) as u64;
        let self_fingerprint = self
            .app_state
            .read(cx)
            .client_info
            .as_ref()
            .map(|c| c.token.clone());
        let announce_info = self.app_state.read(cx).client_info.clone();
        let use_https = self.settings_state.encryption;
        let handle = self.app_state.read(cx).tokio_handle.clone();
        let join = handle.spawn(async move {
            if let Some(info) = announce_info {
                if let Err(err) = crate::core::multicast::send_multicast_announcement(
                    info.alias,
                    info.token,
                    port,
                    use_https,
                    info.device_model,
                    info.device_type,
                )
                .await
                {
                    log::debug!("send multicast announcement failed: {}", err);
                }
            }
            crate::core::discovery::scan_local_network(
                port,
                use_https,
                Duration::from_millis(timeout_ms),
                self_fingerprint,
            )
            .await
        });

        cx.spawn(async move |this, cx| {
            let discovered = match join.await {
                Ok(items) => items,
                Err(err) => {
                    log::error!("discovery scan task failed: {}", err);
                    Vec::new()
                }
            };

            let _ = this.update(cx, |this, cx| {
                let mut endpoint_map = this.send_state.nearby_endpoints.clone();
                for d in &discovered {
                    let key = Self::normalized_peer_token(&d.info, &d.ip, d.port);
                    endpoint_map.insert(
                        key,
                        send_state::DeviceEndpoint {
                            ip: d.ip.clone(),
                            port: d.port,
                            https: d.https,
                        },
                    );
                }
                this.send_state.nearby_endpoints = endpoint_map;

                let mut info_map = std::collections::HashMap::<String, ClientInfo>::new();
                for info in this.send_state.nearby_devices.iter().cloned() {
                    info_map.insert(info.token.clone(), info);
                }
                let mut favorites_changed = false;
                for d in &discovered {
                    let normalized = Self::normalize_peer_info(d.info.clone(), &d.ip, d.port);
                    favorites_changed |= this.send_state.sync_favorite_from_discovered(
                        &normalized.token,
                        &normalized.alias,
                        &d.ip,
                        d.port,
                        d.https,
                    );
                    info_map.insert(normalized.token.clone(), normalized);
                }
                if favorites_changed {
                    this.send_state.persist_favorites_if_dirty();
                }
                this.send_state.nearby_devices = info_map.into_values().collect();
                this.send_state
                    .nearby_devices
                    .sort_by(|a, b| a.alias.cmp(&b.alias));

                for d in discovered.iter() {
                    crate::core::discovery::register_passive_device(
                        crate::core::discovery::DiscoveredDevice {
                            info: d.info.clone(),
                            ip: d.ip.clone(),
                            port: d.port,
                            https: d.https,
                        },
                    );
                }
                this.send_state.scanning = false;
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn ensure_has_selected_files(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.send_state.selected_files.is_empty() {
            self.open_simple_notice_dialog("请先选择要发送的文件或文本", window, cx);
            false
        } else {
            true
        }
    }

    pub(super) fn open_simple_notice_dialog(
        &self,
        message: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let msg = message.to_string();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            dialog
                .title("提示")
                .overlay(true)
                .w(px(320.))
                .child(div().w_full().text_sm().child(msg.clone()))
                .alert()
                .button_props(gpui_component::dialog::DialogButtonProps::default().ok_text("确定"))
        });
    }

    pub(super) fn open_favorites_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let favorites = self.send_state.favorite_list_sorted();
        let home_entity = cx.entity();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let home_for_add = home_entity.clone();
            dialog
                .title("收藏夹")
                .overlay(true)
                .w(px(360.))
                .child(
                    v_flex()
                        .w_full()
                        .gap(px(10.))
                        .when(favorites.is_empty(), |this| {
                            this.child(
                                div()
                                    .w_full()
                                    .text_sm()
                                    .text_color(_cx.theme().muted_foreground)
                                    .child("暂无收藏设备，可手动添加。"),
                            )
                        })
                        .when(!favorites.is_empty(), |this| {
                            this.children(favorites.iter().map(|favorite| {
                                let favorite = favorite.clone();
                                let home_for_pick = home_entity.clone();
                                let home_for_edit = home_entity.clone();
                                let home_for_delete = home_entity.clone();
                                let favorite_for_edit = favorite.clone();
                                let favorite_for_delete = favorite.clone();
                                let row_alias = favorite.alias.clone();
                                let row_ip = favorite.ip.clone();
                                let row_port = favorite.port;
                                let send_ip = row_ip.clone();
                                let send_alias = row_alias.clone();
                                let send_token = favorite.token.clone();
                                let send_https = favorite.https;
                                let edit_id_token = favorite.token.clone();
                                let delete_id_token = favorite.token.clone();
                                v_flex()
                                    .w_full()
                                    .gap(px(6.))
                                    .child(
                                        Button::new(format!(
                                            "favorite-device-send-{}-{}",
                                            row_ip, row_port
                                        ))
                                        .custom(
                                            ButtonCustomVariant::new(_cx)
                                                .color(_cx.theme().secondary)
                                                .foreground(_cx.theme().foreground)
                                                .hover(_cx.theme().secondary)
                                                .active(_cx.theme().secondary),
                                        )
                                        .w_full()
                                        .h(px(48.))
                                        .rounded_md()
                                        .on_click(move |_event, window, cx| {
                                            window.close_dialog(cx);
                                            home_for_pick.update(cx, |this, cx| {
                                                this.send_state.target_device = None;
                                                this.send_to_favorite_device(
                                                    send_state::FavoriteDevice {
                                                        token: send_token.clone(),
                                                        alias: send_alias.clone(),
                                                        ip: send_ip.clone(),
                                                        port: row_port,
                                                        https: send_https,
                                                        custom_alias: favorite.custom_alias,
                                                    },
                                                    window,
                                                    cx,
                                                );
                                            });
                                        })
                                        .child(
                                            h_flex()
                                                .w_full()
                                                .justify_between()
                                                .items_center()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_medium()
                                                        .child(row_alias.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(_cx.theme().muted_foreground)
                                                        .child(format!("{}:{}", row_ip, row_port)),
                                                ),
                                        ),
                                    )
                                    .child(
                                        h_flex()
                                            .w_full()
                                            .justify_end()
                                            .gap(px(8.))
                                            .child(
                                                Button::new(format!(
                                                    "favorite-device-edit-{}",
                                                    edit_id_token
                                                ))
                                                .ghost()
                                                .on_click(move |_event, window, cx| {
                                                    window.close_dialog(cx);
                                                    let preset = favorite_for_edit.clone();
                                                    home_for_edit.update(cx, |this, cx| {
                                                        this.open_edit_favorite_dialog(
                                                            Some(preset.clone()),
                                                            window,
                                                            cx,
                                                        );
                                                    });
                                                })
                                                .child("编辑"),
                                            )
                                            .child(
                                                Button::new(format!(
                                                    "favorite-device-delete-{}",
                                                    delete_id_token
                                                ))
                                                .ghost()
                                                .on_click(move |_event, window, cx| {
                                                    window.close_dialog(cx);
                                                    let token = favorite_for_delete.token.clone();
                                                    let alias = favorite_for_delete.alias.clone();
                                                    home_for_delete.update(cx, |this, cx| {
                                                        this.open_confirm_remove_favorite_dialog(
                                                            token.clone(),
                                                            alias.clone(),
                                                            window,
                                                            cx,
                                                        );
                                                    });
                                                })
                                                .child("删除"),
                                            ),
                                    )
                            }))
                        })
                        .child(
                            Button::new("favorites-add-manual")
                                .primary()
                                .w_full()
                                .on_click(move |_event, window, cx| {
                                    window.close_dialog(cx);
                                    home_for_add.update(cx, |this, cx| {
                                        this.open_edit_favorite_dialog(None, window, cx);
                                    });
                                })
                                .child("手动添加收藏设备"),
                        ),
                )
                .alert()
                .button_props(gpui_component::dialog::DialogButtonProps::default().ok_text("关闭"))
        });
    }

    pub(super) fn open_edit_favorite_dialog(
        &mut self,
        preset: Option<send_state::FavoriteDevice>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let alias_input = cx.new(|cx| InputState::new(window, cx).placeholder("设备名称（可选）"));
        let ip_input = cx.new(|cx| InputState::new(window, cx).placeholder("IP 地址"));
        let port_input = cx.new(|cx| InputState::new(window, cx).placeholder("端口（默认 53317）"));
        if let Some(ref item) = preset {
            alias_input.update(cx, |state, cx| {
                state.set_value(item.alias.clone(), window, cx);
            });
            ip_input.update(cx, |state, cx| {
                state.set_value(item.ip.clone(), window, cx);
            });
            port_input.update(cx, |state, cx| {
                state.set_value(item.port.to_string(), window, cx);
            });
        } else {
            port_input.update(cx, |state, cx| {
                state.set_value("53317", window, cx);
            });
        }
        let home_entity = cx.entity();

        window.open_dialog(cx, move |dialog, _window, _cx| {
            let alias_for_ok = alias_input.clone();
            let ip_for_ok = ip_input.clone();
            let port_for_ok = port_input.clone();
            let home_for_ok = home_entity.clone();
            let preset_for_ok = preset.clone();
            let original_alias = preset
                .as_ref()
                .map(|item| item.alias.clone())
                .unwrap_or_default();
            let original_custom_alias = preset
                .as_ref()
                .map(|item| item.custom_alias)
                .unwrap_or(false);
            dialog
                .title(if preset.is_some() {
                    "编辑收藏设备"
                } else {
                    "添加收藏设备"
                })
                .overlay(true)
                .w(px(360.))
                .child(
                    v_flex()
                        .w_full()
                        .gap(px(10.))
                        .child(Input::new(&alias_input).appearance(true))
                        .child(Input::new(&ip_input).appearance(true))
                        .child(Input::new(&port_input).appearance(true)),
                )
                .confirm()
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("下一步")
                        .cancel_text("取消"),
                )
                .on_ok(move |_event, window, cx| {
                    let alias = alias_for_ok.read(cx).value().trim().to_string();
                    let ip = ip_for_ok.read(cx).value().trim().to_string();
                    let raw_port = port_for_ok.read(cx).value().trim().to_string();
                    if ip.is_empty() {
                        home_for_ok.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("IP 地址不能为空", window, cx);
                        });
                        return false;
                    }
                    let Ok(port) = raw_port.parse::<u16>() else {
                        home_for_ok.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("端口必须是 1-65535 的数字", window, cx);
                        });
                        return false;
                    };
                    if port == 0 {
                        home_for_ok.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("端口必须是 1-65535 的数字", window, cx);
                        });
                        return false;
                    }
                    let token = if let Some(item) = &preset_for_ok {
                        item.token.clone()
                    } else {
                        format!("manual:{}:{}", ip, port)
                    };
                    let display_alias = if alias.is_empty() {
                        ip.clone()
                    } else {
                        alias.clone()
                    };
                    let https = preset_for_ok
                        .as_ref()
                        .map(|item| item.https)
                        .unwrap_or(false);
                    let custom_alias = if alias.is_empty() {
                        false
                    } else if original_alias.is_empty() {
                        true
                    } else {
                        original_custom_alias || alias != original_alias
                    };
                    window.close_dialog(cx);
                    home_for_ok.update(cx, |this, cx| {
                        this.open_confirm_add_favorite_dialog(
                            token,
                            display_alias,
                            ip,
                            port,
                            https,
                            custom_alias,
                            window,
                            cx,
                        );
                    });
                    true
                })
        });
    }

    fn send_to_favorite_device(
        &mut self,
        favorite: send_state::FavoriteDevice,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let home_entity = cx.entity();
        let window_handle = window.window_handle();
        let tokio_handle = self.app_state.read(cx).tokio_handle.clone();
        let self_fingerprint = self
            .app_state
            .read(cx)
            .client_info
            .as_ref()
            .map(|v| v.token.clone());
        let favorite_for_probe = favorite.clone();
        let join = tokio_handle.spawn(async move {
            crate::core::discovery::probe_device(
                &favorite_for_probe.ip,
                favorite_for_probe.port,
                favorite_for_probe.https,
                self_fingerprint,
            )
            .await
        });
        cx.spawn(async move |_this, cx| {
            let probe_result = match join.await {
                Ok(item) => item,
                Err(err) => {
                    log::warn!("favorite probe task failed: {}", err);
                    None
                }
            };
            let _ = window_handle.update(cx, |_, window, cx| {
                let _ = home_entity.update(cx, |this, cx| {
                    if probe_result.is_none() {
                        this.open_simple_notice_dialog(
                            "收藏设备暂不可达，请确认设备在线并在同一网络。",
                            window,
                            cx,
                        );
                        return;
                    }
                    this.execute_send(favorite.ip.clone(), favorite.port, window, cx);
                });
            });
        })
        .detach();
    }

    pub(super) fn open_confirm_remove_favorite_dialog(
        &mut self,
        token: String,
        alias: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let home_entity = cx.entity();
        let display_name = if alias.trim().is_empty() {
            "该设备".to_string()
        } else {
            alias
        };
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let home_for_ok = home_entity.clone();
            let token_for_ok = token.clone();
            dialog
                .title("删除收藏")
                .overlay(true)
                .w(px(340.))
                .child(
                    div()
                        .w_full()
                        .text_sm()
                        .child(format!("确认将 \"{}\" 从收藏夹移除吗？", display_name)),
                )
                .confirm()
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("删除")
                        .cancel_text("取消"),
                )
                .on_ok(move |_event, _window, cx| {
                    home_for_ok.update(cx, |this, _cx| {
                        this.send_state.remove_favorite_device(&token_for_ok);
                    });
                    true
                })
        });
    }

    pub(super) fn open_confirm_add_favorite_dialog(
        &mut self,
        token: String,
        alias: String,
        ip: String,
        port: u16,
        https: bool,
        custom_alias: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let home_entity = cx.entity();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let home_for_ok = home_entity.clone();
            let alias_text = if alias.trim().is_empty() {
                ip.clone()
            } else {
                alias.clone()
            };
            let token_for_ok = token.clone();
            let alias_for_ok = alias_text.clone();
            let ip_for_ok = ip.clone();
            dialog
                .title("确认添加收藏")
                .overlay(true)
                .w(px(360.))
                .child(
                    v_flex()
                        .w_full()
                        .gap(px(8.))
                        .child(div().text_sm().child(format!("设备名：{}", alias_text)))
                        .child(div().text_sm().child(format!("地址：{}:{}", ip, port)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(_cx.theme().muted_foreground)
                                .child("请确认设备信息后再添加。"),
                        ),
                )
                .confirm()
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("添加")
                        .cancel_text("取消"),
                )
                .on_ok(move |_event, _window, cx| {
                    home_for_ok.update(cx, |this, _cx| {
                        this.send_state.add_or_update_favorite_device(
                            token_for_ok.clone(),
                            alias_for_ok.clone(),
                            ip_for_ok.clone(),
                            port,
                            https,
                            custom_alias,
                        );
                    });
                    true
                })
        });
    }

    pub(super) fn handle_pick_content(
        &mut self,
        content_type: SendContentType,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.send_state.send_content_type = content_type;
        match content_type {
            SendContentType::Text => self.open_text_input_dialog(window, cx),
            SendContentType::File => self.handle_pick_from_system(false, window, cx),
            SendContentType::Folder => self.handle_pick_from_system(true, window, cx),
            SendContentType::Media => self.handle_pick_clipboard(window, cx),
        }
    }

    fn handle_pick_from_system(
        &mut self,
        pick_folder: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let window_handle = window.window_handle();
        let home_entity = cx.entity();
        let send_selection_state = self.send_selection_state.clone();
        let tokio_handle = self.app_state.read(cx).tokio_handle.clone();
        let join = tokio_handle.spawn(async move {
            let uris = if pick_folder {
                crate::platform::file_picker::pick_folders().await
            } else {
                crate::platform::file_picker::pick_files().await
            };
            match uris {
                Ok(uris) => {
                    if uris.is_empty() {
                        PathPickOutcome::Cancelled
                    } else {
                        let paths = uris
                            .into_iter()
                            .filter_map(|uri| {
                                crate::platform::file_picker::picker_uri_to_path(&uri)
                            })
                            .collect::<Vec<_>>();
                        if paths.is_empty() {
                            PathPickOutcome::Failed
                        } else {
                            PathPickOutcome::Success(paths)
                        }
                    }
                }
                Err(err) => {
                    log::error!("pick from system failed: {}", err);
                    PathPickOutcome::Failed
                }
            }
        });

        cx.spawn(async move |_this, cx| {
            let outcome = match join.await {
                Ok(outcome) => outcome,
                Err(err) => {
                    log::error!("picker task failed: {}", err);
                    PathPickOutcome::Failed
                }
            };
            match outcome {
                PathPickOutcome::Success(paths) => {
                    let mut added = 0usize;
                    let _ = send_selection_state.update(cx, |state, _| {
                        added = state.add_paths_recursive(paths.clone());
                    });
                    if added > 0 {
                        return;
                    }
                    let _ = window_handle.update(cx, |_, window, cx| {
                        let _ = home_entity.update(cx, |this, cx| {
                            this.open_simple_notice_dialog(
                                "未添加到可发送文件，请确认已授权并且文件可读。",
                                window,
                                cx,
                            );
                        });
                    });
                }
                PathPickOutcome::Cancelled => {}
                PathPickOutcome::Failed => {
                    let _ = window_handle.update(cx, |_, window, cx| {
                        let _ = home_entity.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("选择文件失败。", window, cx);
                        });
                    });
                }
            }
        })
        .detach();
    }

    fn handle_pick_clipboard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let window_handle = window.window_handle();
        let home_entity = cx.entity();
        let tokio_handle = self.app_state.read(cx).tokio_handle.clone();

        let join = tokio_handle.spawn(async move {
            let permission_granted =
                match crate::platform::clipboard::ensure_read_clipboard_permission().await {
                    Ok(granted) => granted,
                    Err(err) => {
                        log::error!("ensure read clipboard permission failed: {}", err);
                        false
                    }
                };
            if !permission_granted {
                return ClipboardPickOutcome::PermissionDenied;
            }

            let text = match crate::platform::clipboard::read_clipboard_text().await {
                Ok(text) => text,
                Err(err) => {
                    log::error!("read clipboard text failed: {}", err);
                    return ClipboardPickOutcome::ReadFailed;
                }
            };
            if text.is_empty() {
                return ClipboardPickOutcome::Success(String::new());
            }

            ClipboardPickOutcome::Success(text)
        });

        cx.spawn(async move |_this, cx| {
            let outcome = match join.await {
                Ok(outcome) => outcome,
                Err(err) => {
                    log::error!("clipboard task failed: {}", err);
                    ClipboardPickOutcome::ReadFailed
                }
            };

            match outcome {
                ClipboardPickOutcome::Success(text) => {
                    if text.is_empty() {
                        return;
                    }
                    let _ = home_entity.update(cx, |this, cx| {
                        this.send_selection_state.update(cx, |state, _| {
                            state.add_text(text.clone());
                        });
                    });
                }
                ClipboardPickOutcome::PermissionDenied => {
                    let _ = window_handle.update(cx, |_, window, cx| {
                        let _ = home_entity.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("无权限。请开启权限。", window, cx);
                        });
                    });
                }
                ClipboardPickOutcome::ReadFailed => {
                    let _ = window_handle.update(cx, |_, window, cx| {
                        let _ = home_entity.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("读取剪贴板失败。", window, cx);
                        });
                    });
                }
            }
        })
        .detach();
    }

    pub(super) fn open_add_content_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let home_entity = cx.entity();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let home_file = home_entity.clone();
            let home_folder = home_entity.clone();
            let home_text = home_entity.clone();
            let home_clipboard = home_entity.clone();
            let variant = ButtonCustomVariant::new(_cx)
                .color(_cx.theme().secondary)
                .foreground(_cx.theme().foreground)
                .hover(_cx.theme().secondary)
                .active(_cx.theme().secondary);
            dialog
                .title("你想加入什么文件？")
                .overlay(true)
                .w(px(340.))
                .child(
                    h_flex()
                        .w_full()
                        .gap(px(10.))
                        .flex_wrap()
                        .justify_start()
                        .child(
                            Button::new("add-file")
                                .custom(variant.clone())
                                .w(px(90.))
                                .h(px(65.))
                                .rounded_md()
                                .on_click(move |_event, window, cx| {
                                    window.close_dialog(cx);
                                    home_file.update(cx, |this, cx| {
                                        this.handle_pick_content(SendContentType::File, window, cx);
                                    });
                                })
                                .child(
                                    v_flex()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(4.))
                                        .child(
                                            Icon::default()
                                                .path("icons/file.svg")
                                                .with_size(gpui_component::Size::Medium)
                                                .text_color(_cx.theme().foreground),
                                        )
                                        .child(div().text_sm().text_center().child("文件")),
                                ),
                        )
                        .child(
                            Button::new("add-folder")
                                .custom(variant.clone())
                                .w(px(90.))
                                .h(px(65.))
                                .rounded_md()
                                .on_click(move |_event, window, cx| {
                                    window.close_dialog(cx);
                                    home_folder.update(cx, |this, cx| {
                                        this.handle_pick_content(
                                            SendContentType::Folder,
                                            window,
                                            cx,
                                        );
                                    });
                                })
                                .child(
                                    v_flex()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(4.))
                                        .child(
                                            Icon::default()
                                                .path("icons/folder.svg")
                                                .with_size(gpui_component::Size::Medium)
                                                .text_color(_cx.theme().foreground),
                                        )
                                        .child(div().text_sm().text_center().child("文件夹")),
                                ),
                        )
                        .child(
                            Button::new("add-text")
                                .custom(variant.clone())
                                .w(px(90.))
                                .h(px(65.))
                                .rounded_md()
                                .on_click(move |_event, window, cx| {
                                    window.close_dialog(cx);
                                    home_text.update(cx, |this, cx| {
                                        this.handle_pick_content(SendContentType::Text, window, cx);
                                    });
                                })
                                .child(
                                    v_flex()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(4.))
                                        .child(
                                            Icon::default()
                                                .path("icons/book-open.svg")
                                                .with_size(gpui_component::Size::Medium)
                                                .text_color(_cx.theme().foreground),
                                        )
                                        .child(div().text_sm().text_center().child("文本")),
                                ),
                        )
                        .child(
                            Button::new("add-clipboard")
                                .custom(variant)
                                .w(px(90.))
                                .h(px(65.))
                                .rounded_md()
                                .on_click(move |_event, window, cx| {
                                    window.close_dialog(cx);
                                    home_clipboard.update(cx, |this, cx| {
                                        this.handle_pick_content(
                                            SendContentType::Media,
                                            window,
                                            cx,
                                        );
                                    });
                                })
                                .child(
                                    v_flex()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(4.))
                                        .child(
                                            Icon::default()
                                                .path("icons/external-link.svg")
                                                .with_size(gpui_component::Size::Medium)
                                                .text_color(_cx.theme().foreground),
                                        )
                                        .child(div().text_sm().text_center().child("剪贴板")),
                                ),
                        ),
                )
                .alert()
                .button_props(gpui_component::dialog::DialogButtonProps::default().ok_text("关闭"))
        });
    }

    pub(super) fn open_send_mode_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let current_mode = self.send_state.send_mode;
        let home_entity = cx.entity();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let home_single = home_entity.clone();
            let home_multiple = home_entity.clone();
            let home_link = home_entity.clone();
            dialog
                .title("发送模式")
                .overlay(true)
                .w(px(320.))
                .child(
                    v_flex()
                        .w_full()
                        .gap(px(8.))
                        .child(
                            Button::new("send-mode-single")
                                .with_variant(gpui_component::button::ButtonVariant::Secondary)
                                .outline()
                                .w_full()
                                .on_click(move |_event, window, cx| {
                                    let _ = home_single.update(cx, |this, _| {
                                        this.apply_send_mode_default(SendMode::Single);
                                    });
                                    window.close_dialog(cx);
                                })
                                .child(
                                    h_flex()
                                        .w_full()
                                        .justify_between()
                                        .items_center()
                                        .child(div().text_sm().child("单接收者"))
                                        .child(if matches!(current_mode, SendMode::Single) {
                                            Icon::default()
                                                .path("icons/check.svg")
                                                .with_size(Size::Small)
                                        } else {
                                            Icon::default()
                                                .path("icons/more-horizontal.svg")
                                                .with_size(Size::Small)
                                                .text_color(_cx.theme().muted_foreground)
                                        }),
                                ),
                        )
                        .child(
                            Button::new("send-mode-multiple")
                                .with_variant(gpui_component::button::ButtonVariant::Secondary)
                                .outline()
                                .w_full()
                                .on_click(move |_event, window, cx| {
                                    let _ = home_multiple.update(cx, |this, _| {
                                        this.apply_send_mode_default(SendMode::Multiple);
                                    });
                                    window.close_dialog(cx);
                                })
                                .child(
                                    h_flex()
                                        .w_full()
                                        .justify_between()
                                        .items_center()
                                        .child(div().text_sm().child("多个接收者"))
                                        .child(if matches!(current_mode, SendMode::Multiple) {
                                            Icon::default()
                                                .path("icons/check.svg")
                                                .with_size(Size::Small)
                                        } else {
                                            Icon::default()
                                                .path("icons/more-horizontal.svg")
                                                .with_size(Size::Small)
                                                .text_color(_cx.theme().muted_foreground)
                                        }),
                                ),
                        )
                        .child(
                            Button::new("send-mode-link")
                                .with_variant(gpui_component::button::ButtonVariant::Secondary)
                                .outline()
                                .w_full()
                                .on_click(move |_event, window, cx| {
                                    let mut can_open = false;
                                    let _ = home_link.update(cx, |this, cx| {
                                        if !this.ensure_has_selected_files(window, cx) {
                                            return;
                                        }
                                        this.apply_send_mode_default(SendMode::Link);
                                        can_open = true;
                                    });
                                    if can_open {
                                        window.close_dialog(cx);
                                        RouterState::global_mut(cx).location.pathname =
                                            "/send/link".into();
                                        window.refresh();
                                    }
                                })
                                .child(
                                    h_flex()
                                        .w_full()
                                        .justify_between()
                                        .items_center()
                                        .child(div().text_sm().child("通过分享链接发送"))
                                        .child(if matches!(current_mode, SendMode::Link) {
                                            Icon::default()
                                                .path("icons/check.svg")
                                                .with_size(Size::Small)
                                        } else {
                                            Icon::default()
                                                .path("icons/more-horizontal.svg")
                                                .with_size(Size::Small)
                                                .text_color(_cx.theme().muted_foreground)
                                        }),
                                ),
                        ),
                )
                .alert()
                .button_props(gpui_component::dialog::DialogButtonProps::default().ok_text("关闭"))
        });
    }

    pub(super) fn cycle_send_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let next_mode = match self.send_state.send_mode {
            SendMode::Single => SendMode::Multiple,
            SendMode::Multiple => SendMode::Link,
            SendMode::Link => SendMode::Single,
        };
        let mode_text = match next_mode {
            SendMode::Single => "单设备发送模式",
            SendMode::Multiple => "多设备发送模式（基础）",
            SendMode::Link => "链接分享模式",
        };
        if matches!(next_mode, SendMode::Link) && !self.ensure_has_selected_files(window, cx) {
            return;
        }
        self.apply_send_mode_default(next_mode);
        if matches!(next_mode, SendMode::Link) {
            RouterState::global_mut(cx).location.pathname = "/send/link".into();
            window.refresh();
        } else {
            self.open_simple_notice_dialog(mode_text, window, cx);
        }
    }

    fn open_share_link_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut entries = Vec::new();
        for file in &self.send_state.selected_files {
            if let Some(text) = &file.text_content {
                entries.push(crate::core::share_links::SharedEntry::Text {
                    name: if file.name.is_empty() {
                        "message.txt".to_string()
                    } else {
                        file.name.clone()
                    },
                    content: text.clone(),
                });
            } else {
                entries.push(crate::core::share_links::SharedEntry::File {
                    name: file.name.clone(),
                    path: file.path.clone(),
                    file_type: file.file_type.clone(),
                });
            }
        }
        let Some(share_id) = crate::core::share_links::create_share(entries) else {
            self.open_simple_notice_dialog("创建分享链接失败。", window, cx);
            return;
        };
        let scheme = if self.settings_state.encryption {
            "https"
        } else {
            "http"
        };
        let host = if let Some(ip) = detect_primary_route_ipv4() {
            ip.to_string()
        } else if let Some(ip) = self.send_state.local_ips.first() {
            ip.clone()
        } else {
            "127.0.0.1".to_string()
        };
        let link = format!(
            "{}://{}:{}/share/{}",
            scheme, host, self.settings_state.server_port, share_id
        );

        let home_entity = cx.entity();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let home_for_copy = home_entity.clone();
            let link_for_copy = link.clone();
            dialog
                .title("分享链接")
                .overlay(true)
                .w(px(380.))
                .child(
                    v_flex()
                        .w_full()
                        .gap(px(10.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(_cx.theme().muted_foreground)
                                .child("将下方链接分享给对方浏览器下载："),
                        )
                        .child(
                            div()
                                .w_full()
                                .rounded_md()
                                .px(px(10.))
                                .py(px(8.))
                                .bg(_cx.theme().muted)
                                .text_sm()
                                .child(link.clone()),
                        )
                        .child(
                            Button::new("share-link-copy")
                                .primary()
                                .on_click(move |_event, window, cx| {
                                    let link_text = link_for_copy.clone();
                                    home_for_copy.update(cx, |this, cx| {
                                        let window_handle = window.window_handle();
                                        let home_entity = cx.entity();
                                        let tokio_handle =
                                            this.app_state.read(cx).tokio_handle.clone();
                                        let join = tokio_handle.spawn(async move {
                                            crate::platform::clipboard::write_clipboard_text(
                                                link_text,
                                            )
                                            .await
                                            .unwrap_or(false)
                                        });
                                        cx.spawn(async move |_this, cx| {
                                            let copied = join.await.unwrap_or(false);
                                            let _ = window_handle.update(cx, |_, window, cx| {
                                                let _ = home_entity.update(cx, |this, cx| {
                                                    if copied {
                                                        this.open_simple_notice_dialog(
                                                            "链接已复制到剪贴板。",
                                                            window,
                                                            cx,
                                                        );
                                                    } else {
                                                        this.open_simple_notice_dialog(
                                                            "复制失败，请手动复制链接。",
                                                            window,
                                                            cx,
                                                        );
                                                    }
                                                });
                                            });
                                        })
                                        .detach();
                                    });
                                })
                                .child("复制链接"),
                        ),
                )
                .alert()
                .button_props(gpui_component::dialog::DialogButtonProps::default().ok_text("关闭"))
        });
    }

    fn init_select_states(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Theme select: 系统 / 浅色 / 深色
        let theme_idx = match self.settings_state.theme_mode {
            ThemeMode::System => 0,
            ThemeMode::Light => 1,
            ThemeMode::Dark => 2,
        };
        let theme_select = cx.new(|cx| {
            SelectState::new(
                vec!["系统", "浅色", "深色"],
                Some(IndexPath::default().row(theme_idx)),
                window,
                cx,
            )
        });
        cx.subscribe_in(
            &theme_select,
            window,
            |this, _, event: &SelectEvent<Vec<&'static str>>, _win, _cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.settings_state.theme_mode = match *value {
                        "浅色" => ThemeMode::Light,
                        "深色" => ThemeMode::Dark,
                        _ => ThemeMode::System,
                    };
                    this.persist_settings();
                }
            },
        )
        .detach();
        self.theme_select = Some(theme_select);

        // Color select: 系统 / NearSend / OLED
        let color_idx = match self.settings_state.color_mode {
            ColorMode::System => 0,
            ColorMode::LocalSend => 1,
            ColorMode::Oled => 2,
        };
        let color_select = cx.new(|cx| {
            SelectState::new(
                vec!["系统", "NearSend", "OLED"],
                Some(IndexPath::default().row(color_idx)),
                window,
                cx,
            )
        });
        cx.subscribe_in(
            &color_select,
            window,
            |this, _, event: &SelectEvent<Vec<&'static str>>, _win, _cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.settings_state.color_mode = match *value {
                        "NearSend" => ColorMode::LocalSend,
                        "OLED" => ColorMode::Oled,
                        _ => ColorMode::System,
                    };
                    this.persist_settings();
                }
            },
        )
        .detach();
        self.color_select = Some(color_select);

        // Language select
        let language_select = cx.new(|cx| {
            SelectState::new(
                vec!["系统", "简体中文", "English", "日本語"],
                Some(IndexPath::default()),
                window,
                cx,
            )
        });
        cx.subscribe_in(
            &language_select,
            window,
            |this, _, event: &SelectEvent<Vec<&'static str>>, _win, _cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.settings_state.language = value.to_string();
                    this.persist_settings();
                }
            },
        )
        .detach();
        self.language_select = Some(language_select);
    }

    /// Opens a dialog with a multiline text input for sending text messages.
    /// Matches LocalSend's MessageInputDialog behavior.
    pub(super) fn open_text_input_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .auto_grow(3, 5)
                .placeholder("输入文本内容")
                .soft_wrap(true)
        });
        self.text_input_state = Some(input_state.clone());

        let home_entity = cx.entity();

        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();

            dialog
                .title("发送文本")
                .overlay(true)
                .w(px(340.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true)),
                )
                .confirm()
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("确认")
                        .cancel_text("取消"),
                )
                .on_ok(move |_event, _window, cx| {
                    let text = input_for_ok.read(cx).value().to_string();
                    if !text.is_empty() {
                        home_for_ok.update(cx, |this, _cx| {
                            this.send_selection_state.update(_cx, |state, _| {
                                state.add_text(text.clone());
                            });
                        });
                    }
                    true
                })
        });
    }

    pub(super) fn open_send_pin_dialog(
        &mut self,
        show_invalid_pin: bool,
        responder: oneshot::Sender<Option<String>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("输入接收端 PIN"));
        let responder = Arc::new(Mutex::new(Some(responder)));
        let home_entity = cx.entity();

        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let responder_for_ok = responder.clone();
            let responder_for_cancel = responder.clone();
            let home_for_ok = home_entity.clone();
            let variant = ButtonCustomVariant::new(_cx)
                .color(_cx.theme().secondary)
                .foreground(_cx.theme().foreground)
                .hover(_cx.theme().secondary)
                .active(_cx.theme().secondary);
            dialog
                .title(if show_invalid_pin {
                    "PIN 错误，请重试"
                } else {
                    "请输入接收端 PIN"
                })
                .overlay(true)
                .w(px(320.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .child(
                    h_flex()
                        .w_full()
                        .justify_end()
                        .gap(px(8.))
                        .child(
                            Button::new("send-pin-cancel")
                                .custom(variant.clone())
                                .on_click(move |_event, window, cx| {
                                    if let Ok(mut guard) = responder_for_cancel.lock() {
                                        if let Some(tx) = guard.take() {
                                            let _ = tx.send(None);
                                        }
                                    }
                                    window.close_dialog(cx);
                                })
                                .child("取消"),
                        )
                        .child(
                            Button::new("send-pin-confirm")
                                .custom(variant)
                                .on_click(move |_event, window, cx| {
                                    let pin = input_for_ok.read(cx).value().trim().to_string();
                                    if pin.is_empty() {
                                        home_for_ok.update(cx, |this, cx| {
                                            this.open_simple_notice_dialog(
                                                "PIN 不能为空",
                                                window,
                                                cx,
                                            );
                                        });
                                        return;
                                    }
                                    if let Ok(mut guard) = responder_for_ok.lock() {
                                        if let Some(tx) = guard.take() {
                                            let _ = tx.send(Some(pin));
                                        }
                                    }
                                    window.close_dialog(cx);
                                })
                                .child("确认"),
                        ),
                )
        });
    }

    pub(super) fn open_server_alias_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("输入设备别名"));
        let current_alias = self.settings_state.server_alias.clone();
        input_state.update(cx, |state, cx| {
            state.set_value(current_alias, window, cx);
        });
        let home_entity = cx.entity();

        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();

            dialog
                .title("编辑别名")
                .overlay(true)
                .w(px(340.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .confirm()
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("保存")
                        .cancel_text("取消"),
                )
                .on_ok(move |_event, window, cx| {
                    let alias = input_for_ok.read(cx).value().trim().to_string();
                    if alias.is_empty() {
                        home_for_ok.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("别名不能为空", window, cx);
                        });
                        return false;
                    }
                    home_for_ok.update(cx, |this, cx| {
                        this.settings_state.server_alias = alias.clone();
                        this.sync_server_config_to_runtime(cx);
                        this.persist_settings();
                    });
                    true
                })
        });
    }

    pub(super) fn open_server_port_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("输入端口号"));
        let current_port = self.settings_state.server_port.to_string();
        input_state.update(cx, |state, cx| {
            state.set_value(current_port, window, cx);
        });
        let home_entity = cx.entity();

        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();

            dialog
                .title("编辑端口")
                .overlay(true)
                .w(px(340.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .confirm()
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("保存")
                        .cancel_text("取消"),
                )
                .on_ok(move |_event, window, cx| {
                    let raw = input_for_ok.read(cx).value().trim().to_string();
                    let Ok(port) = raw.parse::<u16>() else {
                        home_for_ok.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("端口必须是 1-65535 的数字", window, cx);
                        });
                        return false;
                    };
                    if port == 0 {
                        home_for_ok.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("端口必须是 1-65535 的数字", window, cx);
                        });
                        return false;
                    }
                    home_for_ok.update(cx, |this, cx| {
                        this.settings_state.server_port = port;
                        this.sync_server_config_to_runtime(cx);
                        this.persist_settings();
                    });
                    true
                })
        });
    }

    pub(super) fn open_receive_pin_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("输入接收 PIN"));
        let current_pin = self.settings_state.receive_pin.clone();
        input_state.update(cx, |state, cx| {
            state.set_value(current_pin, window, cx);
        });
        let home_entity = cx.entity();

        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();

            dialog
                .title("设置接收 PIN")
                .overlay(true)
                .w(px(340.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .confirm()
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("保存")
                        .cancel_text("取消"),
                )
                .on_ok(move |_event, window, cx| {
                    let pin = input_for_ok.read(cx).value().trim().to_string();
                    if pin.is_empty() {
                        home_for_ok.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("PIN 不能为空", window, cx);
                        });
                        return false;
                    }
                    home_for_ok.update(cx, |this, cx| {
                        this.settings_state.receive_pin = pin.clone();
                        this.sync_server_config_to_runtime(cx);
                        this.persist_settings();
                    });
                    true
                })
        });
    }

    pub(super) fn pick_receive_destination(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let window_handle = window.window_handle();
        let home_entity = cx.entity();
        let tokio_handle = self.app_state.read(cx).tokio_handle.clone();
        let join = tokio_handle
            .spawn(async move { crate::platform::file_picker::pick_save_directory().await });
        cx.spawn(async move |_this, cx| {
            let picked = match join.await {
                Ok(Ok(path)) => path,
                Ok(Err(err)) => {
                    log::error!("pick receive destination failed: {}", err);
                    None
                }
                Err(err) => {
                    log::error!("pick receive destination task failed: {}", err);
                    None
                }
            };
            if let Some(path) = picked {
                let path_text = path.to_string_lossy().to_string();
                let _ = home_entity.update(cx, |this, cx| {
                    this.settings_state.destination = Some(path_text);
                    this.sync_server_config_to_runtime(cx);
                    this.persist_settings();
                });
            } else {
                let _ = window_handle.update(cx, |_, window, cx| {
                    let _ = home_entity.update(cx, |this, cx| {
                        this.open_simple_notice_dialog(
                            "未选择接收目录，保持当前配置。",
                            window,
                            cx,
                        );
                    });
                });
            }
        })
        .detach();
    }

    pub(super) fn clear_receive_destination(&mut self, cx: &mut Context<Self>) {
        self.settings_state.destination = None;
        self.sync_server_config_to_runtime(cx);
        self.persist_settings();
    }

    pub(super) fn cycle_device_type_setting(&mut self, cx: &mut Context<Self>) {
        let next = match normalize_device_type_label(&self.settings_state.device_type).as_str() {
            "mobile" => "Desktop",
            "desktop" => "Web",
            "web" => "Server",
            "server" => "Headless",
            _ => "Mobile",
        };
        self.settings_state.device_type = next.to_string();
        self.sync_server_config_to_runtime(cx);
        self.persist_settings();
    }

    pub(super) fn open_device_model_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入设备型号（可选）"));
        let current = self.settings_state.device_model.clone();
        input_state.update(cx, |state, cx| {
            state.set_value(current, window, cx);
        });
        let home_entity = cx.entity();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();
            dialog
                .title("设备型号")
                .overlay(true)
                .w(px(340.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .confirm()
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("保存")
                        .cancel_text("取消"),
                )
                .on_ok(move |_event, _window, cx| {
                    let value = input_for_ok.read(cx).value().trim().to_string();
                    home_for_ok.update(cx, |this, cx| {
                        this.settings_state.device_model = value;
                        this.sync_server_config_to_runtime(cx);
                        this.persist_settings();
                    });
                    true
                })
        });
    }

    pub(super) fn open_discovery_timeout_dialog(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入发现超时（毫秒）"));
        let current = self.settings_state.discovery_timeout.to_string();
        input_state.update(cx, |state, cx| {
            state.set_value(current, window, cx);
        });
        let home_entity = cx.entity();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();
            dialog
                .title("发现超时")
                .overlay(true)
                .w(px(340.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .confirm()
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("保存")
                        .cancel_text("取消"),
                )
                .on_ok(move |_event, window, cx| {
                    let raw = input_for_ok.read(cx).value().trim().to_string();
                    let Ok(value) = raw.parse::<u32>() else {
                        home_for_ok.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("请输入有效数字", window, cx);
                        });
                        return false;
                    };
                    home_for_ok.update(cx, |this, cx| {
                        this.settings_state.discovery_timeout = value.max(200);
                        this.persist_settings();
                    });
                    true
                })
        });
    }

    pub(super) fn open_multicast_group_dialog(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("输入组播地址"));
        let current = self.settings_state.multicast_group.clone();
        input_state.update(cx, |state, cx| {
            state.set_value(current, window, cx);
        });
        let home_entity = cx.entity();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();
            dialog
                .title("组播地址")
                .overlay(true)
                .w(px(340.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .confirm()
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("保存")
                        .cancel_text("取消"),
                )
                .on_ok(move |_event, window, cx| {
                    let raw = input_for_ok.read(cx).value().trim().to_string();
                    if raw.is_empty() {
                        home_for_ok.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("组播地址不能为空", window, cx);
                        });
                        return false;
                    }
                    home_for_ok.update(cx, |this, cx| {
                        this.settings_state.multicast_group = raw;
                        this.persist_settings();
                    });
                    true
                })
        });
    }

    pub(super) fn cycle_network_filter_mode(&mut self, cx: &mut Context<Self>) {
        self.settings_state.network_filter_mode = match self.settings_state.network_filter_mode {
            NetworkFilterMode::All => NetworkFilterMode::Whitelist,
            NetworkFilterMode::Whitelist => NetworkFilterMode::Blacklist,
            NetworkFilterMode::Blacklist => NetworkFilterMode::All,
        };
        self.sync_server_config_to_runtime(cx);
        self.persist_settings();
    }

    pub(super) fn open_network_filters_dialog(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("每行一个规则：192.168.1.*"));
        let current = self.settings_state.network_filters.join("\n");
        input_state.update(cx, |state, cx| {
            state.set_value(current, window, cx);
        });
        let home_entity = cx.entity();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();
            dialog
                .title("网络接口过滤规则")
                .overlay(true)
                .w(px(380.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .confirm()
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("保存")
                        .cancel_text("取消"),
                )
                .on_ok(move |_event, _window, cx| {
                    let raw = input_for_ok.read(cx).value().to_string();
                    let filters = raw
                        .lines()
                        .map(|line| line.trim().to_string())
                        .filter(|line| !line.is_empty())
                        .collect::<Vec<_>>();
                    home_for_ok.update(cx, |this, cx| {
                        this.settings_state.network_filters = filters;
                        this.sync_server_config_to_runtime(cx);
                        this.persist_settings();
                    });
                    true
                })
        });
    }

    /// Opens LocalSend-like address input dialog (Label/IP tabs, single input).
    pub(super) fn open_send_to_address_dialog(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_send_to_address_dialog_with_mode(AddressInputMode::Label, window, cx);
    }

    fn open_send_to_address_dialog_with_mode(
        &mut self,
        mode: AddressInputMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let placeholder = match mode {
            AddressInputMode::Label => "#123",
            AddressInputMode::IpAddress => "输入IP地址",
        };
        let ip_input_state = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder));
        self.send_ip_input_state = Some(ip_input_state.clone());
        let home_entity = cx.entity();
        let prefixes = self.local_ip_prefixes();
        let example_text = match mode {
            AddressInputMode::Label => {
                if prefixes.is_empty() {
                    "示例：123\n可用网段：\n- 192.168.1.".to_string()
                } else {
                    let mut text = "示例：123\n可用网段：".to_string();
                    for p in prefixes.iter().take(3) {
                        text.push_str(&format!("\n- {}.", p));
                    }
                    text
                }
            }
            AddressInputMode::IpAddress => {
                if prefixes.is_empty() {
                    "示例：\n- 192.168.1.23\n- 192.168.1.123".to_string()
                } else {
                    let mut text = "示例：".to_string();
                    for p in prefixes.iter().take(3) {
                        text.push_str(&format!("\n- {}.123", p));
                    }
                    text
                }
            }
        };
        let tag_tab_style = ButtonCustomVariant::new(cx)
            .color(if mode == AddressInputMode::Label {
                cx.theme().primary.opacity(0.2)
            } else {
                cx.theme().secondary
            })
            .foreground(if mode == AddressInputMode::Label {
                cx.theme().primary
            } else {
                cx.theme().foreground
            })
            .hover(if mode == AddressInputMode::Label {
                cx.theme().primary.opacity(0.2)
            } else {
                cx.theme().secondary
            })
            .active(if mode == AddressInputMode::Label {
                cx.theme().primary.opacity(0.2)
            } else {
                cx.theme().secondary
            });
        let ip_tab_style = ButtonCustomVariant::new(cx)
            .color(if mode == AddressInputMode::IpAddress {
                cx.theme().primary.opacity(0.2)
            } else {
                cx.theme().secondary
            })
            .foreground(if mode == AddressInputMode::IpAddress {
                cx.theme().primary
            } else {
                cx.theme().foreground
            })
            .hover(if mode == AddressInputMode::IpAddress {
                cx.theme().primary.opacity(0.2)
            } else {
                cx.theme().secondary
            })
            .active(if mode == AddressInputMode::IpAddress {
                cx.theme().primary.opacity(0.2)
            } else {
                cx.theme().secondary
            });

        window.open_dialog(cx, move |dialog, _window, _cx| {
            let ip_for_ok = ip_input_state.clone();
            let home_for_ok = home_entity.clone();
            let home_for_tag_tab = home_entity.clone();
            let home_for_ip_tab = home_entity.clone();
            let mode_for_ok = mode;

            dialog
                .title("输入地址")
                .overlay(true)
                .w(px(340.))
                .child(
                    v_flex()
                        .w_full()
                        .gap(px(12.))
                        .child(
                            h_flex()
                                .gap(px(0.))
                                .child(
                                    Button::new("address-mode-label")
                                        .custom(tag_tab_style.clone())
                                        .w(px(72.))
                                        .h(px(32.))
                                        .rounded_l(px(12.))
                                        .rounded_r(px(0.))
                                        .on_click(move |_event, window, cx| {
                                            if mode != AddressInputMode::Label {
                                                window.close_dialog(cx);
                                                home_for_tag_tab.update(cx, |this, cx| {
                                                    this.open_send_to_address_dialog_with_mode(
                                                        AddressInputMode::Label,
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            }
                                        })
                                        .child(div().text_sm().font_medium().child("标签")),
                                )
                                .child(
                                    Button::new("address-mode-ip")
                                        .custom(ip_tab_style.clone())
                                        .w(px(88.))
                                        .h(px(32.))
                                        .rounded_l(px(0.))
                                        .rounded_r(px(12.))
                                        .on_click(move |_event, window, cx| {
                                            if mode != AddressInputMode::IpAddress {
                                                window.close_dialog(cx);
                                                home_for_ip_tab.update(cx, |this, cx| {
                                                    this.open_send_to_address_dialog_with_mode(
                                                        AddressInputMode::IpAddress,
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            }
                                        })
                                        .child(div().text_sm().font_medium().child("IP 地址")),
                                ),
                        )
                        .child(
                            div()
                                .w_full()
                                .shadow_xs()
                                .rounded_md()
                                .child(Input::new(&ip_input_state).appearance(true).large()),
                        )
                        .child(
                            div()
                                .w_full()
                                .text_sm()
                                .text_color(_cx.theme().muted_foreground)
                                .child(example_text.clone()),
                        ),
                )
                .confirm()
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("确认")
                        .cancel_text("取消"),
                )
                .on_ok(move |_event, window, cx| {
                    let raw = ip_for_ok.read(cx).value().trim().to_string();
                    if raw.is_empty() {
                        return false;
                    }
                    home_for_ok.update(cx, |this, cx| {
                        let port = this.settings_state.server_port;
                        match mode_for_ok {
                            AddressInputMode::IpAddress => {
                                this.execute_send(raw.clone(), port, window, cx);
                            }
                            AddressInputMode::Label => {
                                if let Some(ip) = this.resolve_labeled_ip(&raw) {
                                    this.execute_send(ip, port, window, cx);
                                } else {
                                    this.open_simple_notice_dialog(
                                        "无法根据标签推导可用 IP，请切换到“IP 地址”模式直接输入。",
                                        window,
                                        cx,
                                    );
                                }
                            }
                        }
                    });
                    true
                })
        });
    }

    fn resolve_labeled_ip(&self, label: &str) -> Option<String> {
        let suffix = label.trim().trim_start_matches('#');
        let suffix_num: u8 = suffix.parse().ok()?;
        let prefixes = self.local_ip_prefixes();
        let prefix = prefixes.first()?;
        Some(format!("{}.{}", prefix, suffix_num))
    }

    fn local_ip_prefixes(&self) -> Vec<String> {
        let mut prefixes = BTreeSet::new();
        for ip in &self.send_state.local_ips {
            if let Some(p) = ipv4_prefix(ip) {
                prefixes.insert(p);
            }
        }
        if let Some(ip) = detect_primary_route_ipv4() {
            if let Some(p) = ipv4_prefix(&ip.to_string()) {
                prefixes.insert(p);
            }
        }
        prefixes.into_iter().collect()
    }

    pub(super) fn open_send_target_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_send_to_address_dialog(window, cx);
    }

    /// Execute the send flow: build entries from selected_files, spawn thread with tokio runtime.
    pub(super) fn execute_send(
        &mut self,
        ip: String,
        port: u16,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use localsend::http::dto::{ProtocolType, RegisterDto};

        #[derive(Clone)]
        enum SendFileEntry {
            Text {
                content: String,
            },
            File {
                path: std::path::PathBuf,
                name: String,
                size: u64,
                file_type: String,
            },
        }

        let files: Vec<SendFileEntry> = self
            .send_state
            .selected_files
            .iter()
            .map(|f| {
                if let Some(text) = &f.text_content {
                    SendFileEntry::Text {
                        content: text.clone(),
                    }
                } else {
                    SendFileEntry::File {
                        path: f.path.clone(),
                        name: f.name.clone(),
                        size: f.size,
                        file_type: f.file_type.clone(),
                    }
                }
            })
            .collect();
        let is_single_text_message =
            files.len() == 1 && matches!(files.first(), Some(SendFileEntry::Text { .. }));

        if files.is_empty() {
            log::warn!("No files selected to send");
            return;
        }
        self.send_state.pending_send = true;
        self.send_state.session_status = SendSessionStatus::Preparing;
        self.send_state.session_status_text = Some("正在准备发送...".to_string());

        // Build RegisterDto from our client info
        let client_info = self.app_state.read(cx).client_info.clone();
        let our_info = if let Some(info) = client_info {
            RegisterDto {
                alias: info.alias,
                version: info.version,
                device_model: info.device_model,
                device_type: info.device_type,
                token: info.token,
                port: self.receive_state.server_port,
                protocol: ProtocolType::Http,
                has_web_interface: false,
            }
        } else {
            log::error!("No client info available for send");
            return;
        };

        // Grab cert so we can create a fresh LsHttpClient in the transfer task
        let cert = self.app_state.read(cx).cert.clone();
        let tokio_handle = self.app_state.read(cx).tokio_handle.clone();
        let (notify_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel::<SendUiMessage>();
        let window_handle = window.window_handle();
        let home_entity = cx.entity();
        let sent_files_for_history = self.send_state.selected_files.clone();
        let target_device_name = self
            .send_state
            .target_device
            .as_ref()
            .map(|d| d.alias.clone())
            .unwrap_or_else(|| format!("{}:{}", ip, port));
        let history_state = self.history_state.clone();
        log::info!(
            "Starting send to {}:{} with {} files",
            ip,
            port,
            files.len()
        );

        // Spawn on the shared tokio runtime
        tokio_handle.spawn(async move {
            // ===== Compatibility path for official LocalSend app (macOS / mobile): v2 endpoints + v2 DTO =====
            #[derive(Clone, serde::Serialize)]
            #[serde(rename_all = "lowercase")]
            enum V2Protocol {
                Http,
                Https,
            }

            #[derive(Clone, serde::Serialize)]
            #[serde(rename_all = "lowercase")]
            enum V2DeviceType {
                Mobile,
                Desktop,
                Web,
                Headless,
                Server,
            }

            #[derive(Clone, serde::Serialize)]
            #[serde(rename_all = "camelCase")]
            struct V2InfoRegisterDto {
                alias: String,
                version: String,
                #[serde(skip_serializing_if = "Option::is_none")]
                device_model: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                device_type: Option<V2DeviceType>,
                fingerprint: String,
                port: u16,
                protocol: V2Protocol,
                download: bool,
            }

            #[derive(serde::Serialize)]
            #[serde(rename_all = "camelCase")]
            struct V2FileDto {
                id: String,
                file_name: String,
                size: u64,
                file_type: String,
                #[serde(skip_serializing_if = "Option::is_none")]
                preview: Option<String>,
            }

            #[derive(serde::Serialize)]
            #[serde(rename_all = "camelCase")]
            struct V2PrepareUploadRequestDto {
                info: V2InfoRegisterDto,
                files: std::collections::HashMap<String, V2FileDto>,
            }

            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct V2PrepareUploadResponseDto {
                session_id: String,
                files: std::collections::HashMap<String, String>,
            }

            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct V2RegisterResponseDto {
                alias: String,
                version: Option<String>,
                #[serde(default)]
                device_model: Option<String>,
                #[serde(default)]
                device_type: Option<String>,
                #[serde(default)]
                token: Option<String>,
                #[serde(default)]
                fingerprint: Option<String>,
            }

            fn to_v2_device_type(
                device_type: &Option<localsend::model::discovery::DeviceType>,
            ) -> Option<V2DeviceType> {
                match device_type {
                    Some(localsend::model::discovery::DeviceType::Mobile) => {
                        Some(V2DeviceType::Mobile)
                    }
                    Some(localsend::model::discovery::DeviceType::Desktop) => {
                        Some(V2DeviceType::Desktop)
                    }
                    Some(localsend::model::discovery::DeviceType::Web) => Some(V2DeviceType::Web),
                    Some(localsend::model::discovery::DeviceType::Headless) => {
                        Some(V2DeviceType::Headless)
                    }
                    Some(localsend::model::discovery::DeviceType::Server) => {
                        Some(V2DeviceType::Server)
                    }
                    None => None,
                }
            }

            fn from_wire_device_type(
                value: Option<&str>,
            ) -> Option<localsend::model::discovery::DeviceType> {
                match value {
                    Some("mobile") | Some("MOBILE") => {
                        Some(localsend::model::discovery::DeviceType::Mobile)
                    }
                    Some("desktop") | Some("DESKTOP") => {
                        Some(localsend::model::discovery::DeviceType::Desktop)
                    }
                    Some("web") | Some("WEB") => Some(localsend::model::discovery::DeviceType::Web),
                    Some("headless") | Some("HEADLESS") => {
                        Some(localsend::model::discovery::DeviceType::Headless)
                    }
                    Some("server") | Some("SERVER") => {
                        Some(localsend::model::discovery::DeviceType::Server)
                    }
                    _ => None,
                }
            }

            fn remember_peer(peer: crate::core::discovery::DiscoveredDevice) {
                crate::core::discovery::register_passive_device(peer);
            }

            let files_for_v2 = files.clone();
            let mut v2_file_map = std::collections::HashMap::new();
            let mut v2_entry_map = std::collections::HashMap::new();

            for entry in files_for_v2 {
                let file_id = uuid::Uuid::new_v4().to_string();
                let dto = match &entry {
                    SendFileEntry::Text { content } => V2FileDto {
                        id: file_id.clone(),
                        file_name: "message.txt".to_string(),
                        size: content.as_bytes().len() as u64,
                        file_type: "text/plain".to_string(),
                        preview: if is_single_text_message {
                            Some(content.clone())
                        } else {
                            None
                        },
                    },
                    SendFileEntry::File {
                        name,
                        size,
                        file_type,
                        ..
                    } => V2FileDto {
                        id: file_id.clone(),
                        file_name: name.clone(),
                        size: *size,
                        file_type: file_type.clone(),
                        preview: None,
                    },
                };
                v2_file_map.insert(file_id.clone(), dto);
                v2_entry_map.insert(file_id, entry);
            }

            let mut v2_last_err: Option<String> = None;
            let mut v2_no_upload_success = false;
            let mut remembered_peer: Option<crate::core::discovery::DiscoveredDevice> = None;
            let v2_http_client = match reqwest::Client::builder()
                .use_rustls_tls()
                .danger_accept_invalid_certs(true)
                .build()
            {
                Ok(client) => client,
                Err(e) => {
                    log::warn!("failed to create v2 compatibility client: {}", e);
                    // fall through to legacy logic
                    reqwest::Client::new()
                }
            };

            let v2_sender_info = V2InfoRegisterDto {
                alias: our_info.alias.clone(),
                version: our_info.version.clone(),
                device_model: our_info.device_model.clone(),
                device_type: to_v2_device_type(&our_info.device_type),
                // Official LocalSend v2 expects fingerprint field.
                // Reuse token as a stable unique sender identifier.
                fingerprint: our_info.token.clone(),
                port: our_info.port,
                protocol: V2Protocol::Http,
                download: false,
            };

            let v2_payload = V2PrepareUploadRequestDto {
                info: v2_sender_info.clone(),
                files: v2_file_map,
            };

            let v2_protocols = [("http", V2Protocol::Http), ("https", V2Protocol::Https)];
            let mut v2_selected_scheme: Option<&str> = None;
            let mut v2_response: Option<V2PrepareUploadResponseDto> = None;

            for (scheme, _scheme_enum) in v2_protocols {
                let register_url =
                    format!("{}://{}:{}/api/localsend/v2/register", scheme, ip, port);
                match v2_http_client
                    .post(&register_url)
                    .json(&v2_sender_info)
                    .send()
                    .await
                {
                    Ok(res) => {
                        if !res.status().is_success() {
                            log::warn!(
                                "v2 register failed via {} for {}:{} status={}",
                                scheme,
                                ip,
                                port,
                                res.status()
                            );
                        } else {
                            match res.json::<V2RegisterResponseDto>().await {
                                Ok(body) => {
                                    remembered_peer =
                                        Some(crate::core::discovery::DiscoveredDevice {
                                            info: ClientInfo {
                                                alias: body.alias,
                                                version: body
                                                    .version
                                                    .unwrap_or_else(|| "1.0".to_string()),
                                                device_model: body.device_model,
                                                device_type: from_wire_device_type(
                                                    body.device_type.as_deref(),
                                                ),
                                                token: body
                                                    .fingerprint
                                                    .or(body.token)
                                                    .unwrap_or_default(),
                                            },
                                            ip: ip.clone(),
                                            port,
                                            https: scheme == "https",
                                        });
                                }
                                Err(e) => {
                                    log::debug!("decode v2 register response failed: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "v2 register failed via {} for {}:{}: {}",
                            scheme,
                            ip,
                            port,
                            e
                        );
                    }
                }

                let prepare_url = format!(
                    "{}://{}:{}/api/localsend/v2/prepare-upload",
                    scheme, ip, port
                );
                let mut pin: Option<String> = None;
                let mut pin_first_attempt = true;
                loop {
                    let mut req = v2_http_client.post(&prepare_url).json(&v2_payload);
                    if let Some(pin_value) = pin.as_ref() {
                        req = req.query(&[("pin", pin_value)]);
                    }
                    match req.send().await {
                        Ok(res) => {
                            if res.status() == reqwest::StatusCode::NO_CONTENT {
                                // LocalSend message-only transfer: accepted without binary upload.
                                v2_selected_scheme = Some(scheme);
                                v2_no_upload_success = true;
                                break;
                            }
                            if !res.status().is_success() {
                                if res.status() == reqwest::StatusCode::UNAUTHORIZED {
                                    let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                                        status: SendSessionStatus::PinRequired,
                                        message: Some("需要输入接收端 PIN".to_string()),
                                    });
                                    let (pin_tx, pin_rx) = oneshot::channel::<Option<String>>();
                                    let _ = notify_tx.send(SendUiMessage::RequestPin {
                                        show_invalid_pin: !pin_first_attempt,
                                        responder: pin_tx,
                                    });
                                    pin_first_attempt = false;
                                    pin = match pin_rx.await {
                                        Ok(value) => value,
                                        Err(_) => None,
                                    };
                                    if pin.is_none() {
                                        let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                                            status: SendSessionStatus::CancelledByUser,
                                            message: Some("已取消输入 PIN".to_string()),
                                        });
                                        let _ = notify_tx.send(SendUiMessage::Notice(
                                            "已取消输入 PIN。".to_string(),
                                        ));
                                        return;
                                    }
                                    continue;
                                }
                                if matches!(
                                    res.status(),
                                    reqwest::StatusCode::FORBIDDEN
                                        | reqwest::StatusCode::CONFLICT
                                        | reqwest::StatusCode::TOO_MANY_REQUESTS
                                ) {
                                    log::warn!(
                                        "v2 prepare-upload terminal status via {} for {}:{}: {}",
                                        scheme,
                                        ip,
                                        port,
                                        res.status()
                                    );
                                    let _ =
                                        notify_tx.send(SendUiMessage::Notice(match res.status() {
                                            reqwest::StatusCode::FORBIDDEN => {
                                                "对方已拒绝本次传输请求。".to_string()
                                            }
                                            reqwest::StatusCode::CONFLICT => {
                                                "对方当前正忙，请稍后重试。".to_string()
                                            }
                                            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                                                "PIN 尝试次数过多，请稍后再试。".to_string()
                                            }
                                            _ => "发送失败，请稍后重试。".to_string(),
                                        }));
                                    let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                                        status: match res.status() {
                                            reqwest::StatusCode::FORBIDDEN => {
                                                SendSessionStatus::Declined
                                            }
                                            reqwest::StatusCode::CONFLICT => {
                                                SendSessionStatus::RecipientBusy
                                            }
                                            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                                                SendSessionStatus::TooManyAttempts
                                            }
                                            _ => SendSessionStatus::Failed,
                                        },
                                        message: Some(format!("发送失败（{}）", res.status())),
                                    });
                                    return;
                                }
                                let msg = format!("status={}", res.status());
                                log::warn!(
                                    "v2 prepare-upload failed via {} for {}:{}: {}",
                                    scheme,
                                    ip,
                                    port,
                                    msg
                                );
                                v2_last_err = Some(msg);
                                break;
                            }
                            match res.json::<V2PrepareUploadResponseDto>().await {
                                Ok(parsed) => {
                                    v2_selected_scheme = Some(scheme);
                                    v2_response = Some(parsed);
                                    break;
                                }
                                Err(e) => {
                                    let msg =
                                        format!("decode prepare-upload response failed: {}", e);
                                    log::warn!(
                                        "v2 prepare-upload decode failed via {} for {}:{}: {}",
                                        scheme,
                                        ip,
                                        port,
                                        msg
                                    );
                                    v2_last_err = Some(msg);
                                }
                            }
                            break;
                        }
                        Err(e) => {
                            let msg = e.to_string();
                            log::warn!(
                                "v2 prepare-upload failed via {} for {}:{}: {}",
                                scheme,
                                ip,
                                port,
                                msg
                            );
                            v2_last_err = Some(msg);
                            break;
                        }
                    }
                }
                if v2_no_upload_success || v2_response.is_some() {
                    break;
                }
            }
            if v2_no_upload_success {
                if let Some(peer) = remembered_peer.clone() {
                    remember_peer(peer);
                }
                log::info!("v2 message transfer complete (no upload required)");
                let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                    status: SendSessionStatus::Completed,
                    message: Some("发送完成".to_string()),
                });
                return;
            }

            if let (Some(scheme), Some(v2_response)) = (v2_selected_scheme, v2_response) {
                let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                    status: SendSessionStatus::Sending,
                    message: Some("正在发送...".to_string()),
                });
                log::info!(
                    "v2 prepare-upload succeeded for {}:{} over {}, session={}",
                    ip,
                    port,
                    scheme,
                    v2_response.session_id
                );

                if v2_response.files.is_empty() {
                    log::info!(
                        "v2 transfer complete with empty file selection, session_id={}",
                        v2_response.session_id
                    );
                    let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                        status: SendSessionStatus::Completed,
                        message: Some("发送完成".to_string()),
                    });
                    return;
                }
                for (file_id, token) in &v2_response.files {
                    if let Some(entry) = v2_entry_map.remove(file_id) {
                        let (body, content_type) = match entry {
                            SendFileEntry::Text { content } => {
                                (content.into_bytes(), "text/plain".to_string())
                            }
                            SendFileEntry::File {
                                path, file_type, ..
                            } => match tokio::fs::read(&path).await {
                                Ok(data) => (data, file_type),
                                Err(e) => {
                                    log::error!("Failed to read file {:?}: {}", path, e);
                                    continue;
                                }
                            },
                        };

                        let upload_url =
                            format!("{}://{}:{}/api/localsend/v2/upload", scheme, ip, port);
                        let upload_result = v2_http_client
                            .post(&upload_url)
                            .query(&[
                                ("sessionId", v2_response.session_id.as_str()),
                                ("fileId", file_id.as_str()),
                                ("token", token.as_str()),
                            ])
                            .header(reqwest::header::CONTENT_TYPE, content_type)
                            .header(reqwest::header::CONTENT_LENGTH, body.len().to_string())
                            .body(body)
                            .send()
                            .await;

                        match upload_result {
                            Ok(res) if res.status().is_success() => {
                                log::info!("v2 upload succeeded for file_id={}", file_id);
                            }
                            Ok(res) => {
                                log::error!(
                                    "v2 upload failed for file_id={} status={}",
                                    file_id,
                                    res.status()
                                );
                                let _ = notify_tx.send(SendUiMessage::Notice(format!(
                                    "发送失败：文件上传状态异常（{}）。",
                                    res.status()
                                )));
                                let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                                    status: SendSessionStatus::Failed,
                                    message: Some(format!("发送失败（上传状态 {}）", res.status())),
                                });
                            }
                            Err(e) => {
                                log::error!("v2 upload failed for file_id={}: {}", file_id, e);
                                let _ = notify_tx.send(SendUiMessage::Notice(
                                    "发送失败：上传过程中发生网络错误。".to_string(),
                                ));
                                let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                                    status: SendSessionStatus::Failed,
                                    message: Some("发送失败（上传网络错误）".to_string()),
                                });
                            }
                        }
                    }
                }

                log::info!(
                    "v2 transfer complete, session_id={}",
                    v2_response.session_id
                );
                if let Some(peer) = remembered_peer.clone() {
                    remember_peer(peer);
                }
                let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                    status: SendSessionStatus::Completed,
                    message: Some("发送完成".to_string()),
                });
                return;
            } else {
                log::warn!(
                    "v2 send path failed for {}:{}; fallback to legacy core client. last_error={}",
                    ip,
                    port,
                    v2_last_err.unwrap_or_else(|| "unknown".to_string())
                );
            }

            // ===== Legacy fallback path: localsend/core client =====
            // Create a fresh LsHttpClient for this transfer
            let cert = match cert {
                Some(c) => c,
                None => {
                    log::error!("No TLS cert available, cannot create transfer client");
                    let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                        status: SendSessionStatus::Failed,
                        message: Some("发送失败（缺少证书）".to_string()),
                    });
                    let _ = notify_tx.send(SendUiMessage::Notice(
                        "发送失败：本端证书不可用。".to_string(),
                    ));
                    return;
                }
            };
            let client = match localsend::http::client::LsHttpClient::try_new(
                &cert.private_key_pem,
                &cert.cert_pem,
            ) {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to create transfer client: {}", e);
                    let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                        status: SendSessionStatus::Failed,
                        message: Some("发送失败（初始化失败）".to_string()),
                    });
                    let _ = notify_tx.send(SendUiMessage::Notice(
                        "发送失败：无法初始化传输客户端。".to_string(),
                    ));
                    return;
                }
            };

            // Build FileDto map
            let mut file_map = std::collections::HashMap::new();
            let mut entry_map = std::collections::HashMap::new();
            for entry in files {
                let file_id = uuid::Uuid::new_v4().to_string();
                let dto = match &entry {
                    SendFileEntry::Text { content } => localsend::model::transfer::FileDto {
                        id: file_id.clone(),
                        file_name: "message.txt".to_string(),
                        size: content.as_bytes().len() as u64,
                        file_type: "text/plain".to_string(),
                        sha256: None,
                        preview: if is_single_text_message {
                            Some(content.clone())
                        } else {
                            None
                        },
                        metadata: None,
                    },
                    SendFileEntry::File {
                        name,
                        size,
                        file_type,
                        ..
                    } => localsend::model::transfer::FileDto {
                        id: file_id.clone(),
                        file_name: name.clone(),
                        size: *size,
                        file_type: file_type.clone(),
                        sha256: None,
                        preview: None,
                        metadata: None,
                    },
                };
                file_map.insert(file_id.clone(), dto);
                entry_map.insert(file_id, entry);
            }

            let payload = localsend::http::dto::PrepareUploadRequestDto {
                info: our_info,
                files: file_map,
            };

            // Target client protocol can be HTTP or HTTPS.
            // Try HTTP first for backward compatibility, then HTTPS fallback.
            let protocols = [ProtocolType::Http, ProtocolType::Https];
            let mut selected_protocol: Option<ProtocolType> = None;
            let mut response: Option<localsend::http::dto::PrepareUploadResponseDto> = None;
            let mut last_err: Option<String> = None;
            let legacy_http_client = {
                let identity = {
                    let pem = &[
                        cert.cert_pem.as_bytes(),
                        "\n".as_bytes(),
                        cert.private_key_pem.as_bytes(),
                    ]
                    .concat();
                    reqwest::Identity::from_pem(pem)
                };
                match identity.and_then(|id| {
                    reqwest::Client::builder()
                        .use_rustls_tls()
                        .danger_accept_invalid_certs(true)
                        .identity(id)
                        .build()
                }) {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("failed to create legacy reqwest client: {}", e);
                        let _ = notify_tx.send(SendUiMessage::Notice(
                            "发送失败：无法初始化网络客户端。".to_string(),
                        ));
                        let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                            status: SendSessionStatus::Failed,
                            message: Some("发送失败（客户端初始化失败）".to_string()),
                        });
                        return;
                    }
                }
            };

            for protocol in protocols {
                log::info!("Trying protocol={} for {}:{}", protocol.as_str(), ip, port);

                let _public_key = match client
                    .register(&protocol, &ip, port, payload.info.clone())
                    .await
                {
                    Ok(register_result) => {
                        remembered_peer = Some(crate::core::discovery::DiscoveredDevice {
                            info: ClientInfo {
                                alias: register_result.body.alias,
                                version: register_result.body.version,
                                device_model: register_result.body.device_model,
                                device_type: register_result.body.device_type,
                                token: register_result.body.token,
                            },
                            ip: ip.clone(),
                            port,
                            https: matches!(protocol, ProtocolType::Https),
                        });
                        register_result.public_key
                    }
                    Err(e) => {
                        // Some peers may reject/ignore register variants. Keep compatibility by still trying prepare-upload.
                        log::warn!(
                            "register failed via {} for {}:{}: {}",
                            protocol.as_str(),
                            ip,
                            port,
                            e
                        );
                        None
                    }
                };

                let prepare_url = format!(
                    "{}://{}:{}/api/localsend/v3/prepare-upload",
                    protocol.as_str(),
                    ip,
                    port
                );
                let mut pin: Option<String> = None;
                let mut pin_first_attempt = true;
                loop {
                    let mut req = legacy_http_client.post(&prepare_url).json(&payload);
                    if let Some(pin_value) = pin.as_ref() {
                        req = req.query(&[("pin", pin_value)]);
                    }
                    match req.send().await {
                        Ok(res) => {
                            if !res.status().is_success() {
                                if res.status() == reqwest::StatusCode::UNAUTHORIZED {
                                    let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                                        status: SendSessionStatus::PinRequired,
                                        message: Some("需要输入接收端 PIN".to_string()),
                                    });
                                    let (pin_tx, pin_rx) = oneshot::channel::<Option<String>>();
                                    let _ = notify_tx.send(SendUiMessage::RequestPin {
                                        show_invalid_pin: !pin_first_attempt,
                                        responder: pin_tx,
                                    });
                                    pin_first_attempt = false;
                                    pin = match pin_rx.await {
                                        Ok(value) => value,
                                        Err(_) => None,
                                    };
                                    if pin.is_none() {
                                        let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                                            status: SendSessionStatus::CancelledByUser,
                                            message: Some("已取消输入 PIN".to_string()),
                                        });
                                        let _ = notify_tx.send(SendUiMessage::Notice(
                                            "已取消输入 PIN。".to_string(),
                                        ));
                                        return;
                                    }
                                    continue;
                                }

                                if matches!(
                                    res.status(),
                                    reqwest::StatusCode::FORBIDDEN
                                        | reqwest::StatusCode::CONFLICT
                                        | reqwest::StatusCode::TOO_MANY_REQUESTS
                                ) {
                                    let status = res.status();
                                    log::warn!(
                                        "prepare-upload terminal status via {} for {}:{}: {}",
                                        protocol.as_str(),
                                        ip,
                                        port,
                                        status
                                    );
                                    let _ = notify_tx.send(SendUiMessage::Notice(match status {
                                        reqwest::StatusCode::FORBIDDEN => {
                                            "对方已拒绝本次传输请求。".to_string()
                                        }
                                        reqwest::StatusCode::CONFLICT => {
                                            "对方当前正忙，请稍后重试。".to_string()
                                        }
                                        reqwest::StatusCode::TOO_MANY_REQUESTS => {
                                            "PIN 尝试次数过多，请稍后再试。".to_string()
                                        }
                                        _ => "发送失败，请稍后重试。".to_string(),
                                    }));
                                    let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                                        status: match status {
                                            reqwest::StatusCode::FORBIDDEN => {
                                                SendSessionStatus::Declined
                                            }
                                            reqwest::StatusCode::CONFLICT => {
                                                SendSessionStatus::RecipientBusy
                                            }
                                            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                                                SendSessionStatus::TooManyAttempts
                                            }
                                            _ => SendSessionStatus::Failed,
                                        },
                                        message: Some(format!("发送失败（{}）", status)),
                                    });
                                    return;
                                }

                                let msg = format!("status={}", res.status());
                                log::warn!(
                                    "prepare-upload failed via {} for {}:{}: {}",
                                    protocol.as_str(),
                                    ip,
                                    port,
                                    msg
                                );
                                last_err = Some(msg);
                                break;
                            }

                            match res
                                .json::<localsend::http::dto::PrepareUploadResponseDto>()
                                .await
                            {
                                Ok(parsed) => {
                                    selected_protocol = Some(protocol);
                                    response = Some(parsed);
                                    break;
                                }
                                Err(e) => {
                                    let msg =
                                        format!("decode prepare-upload response failed: {}", e);
                                    log::warn!(
                                        "prepare-upload decode failed via {} for {}:{}: {}",
                                        protocol.as_str(),
                                        ip,
                                        port,
                                        msg
                                    );
                                    last_err = Some(msg);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            let msg = e.to_string();
                            log::warn!(
                                "prepare-upload failed via {} for {}:{}: {}",
                                protocol.as_str(),
                                ip,
                                port,
                                msg
                            );
                            last_err = Some(msg);
                            break;
                        }
                    }
                }
                if response.is_some() {
                    break;
                }
            }

            let (protocol, response) = match (selected_protocol, response) {
                (Some(protocol), Some(response)) => (protocol, response),
                _ => {
                    log::error!(
                        "prepare-upload failed for {}:{} after trying HTTP+HTTPS. last_error={}",
                        ip,
                        port,
                        last_err.unwrap_or_else(|| "unknown".to_string())
                    );
                    let _ = notify_tx.send(SendUiMessage::Notice(
                        "发送失败，请检查对端状态后重试。".to_string(),
                    ));
                    let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                        status: SendSessionStatus::Failed,
                        message: Some("发送失败（准备阶段）".to_string()),
                    });
                    return;
                }
            };

            let session_id = response.session_id.clone();
            let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                status: SendSessionStatus::Sending,
                message: Some("正在发送...".to_string()),
            });
            log::info!(
                "Got session_id: {}, protocol: {}, accepted files: {}",
                session_id,
                protocol.as_str(),
                response.files.len()
            );

            // Upload each accepted file
            for (file_id, token) in &response.files {
                if let Some(entry) = entry_map.remove(file_id) {
                    let (tx, rx) = tokio::sync::mpsc::channel::<Vec<u8>>(32);

                    // Feed data into the channel
                    tokio::task::spawn(async move {
                        match entry {
                            SendFileEntry::Text { content } => {
                                let _ = tx.send(content.into_bytes()).await;
                            }
                            SendFileEntry::File { path, .. } => {
                                match tokio::fs::read(&path).await {
                                    Ok(data) => {
                                        let _ = tx.send(data).await;
                                    }
                                    Err(e) => {
                                        log::error!("Failed to read file {:?}: {}", path, e)
                                    }
                                }
                            }
                        }
                    });

                    log::info!("Uploading file_id={}", file_id);
                    if let Err(e) = client
                        .upload(
                            &protocol,
                            &ip,
                            port,
                            session_id.clone(),
                            file_id.clone(),
                            token.clone(),
                            rx,
                        )
                        .await
                    {
                        log::error!("Upload failed for file_id={}: {}", file_id, e);
                        let _ = notify_tx.send(SendUiMessage::Notice(
                            "发送失败：上传过程中发生错误。".to_string(),
                        ));
                        let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                            status: SendSessionStatus::Failed,
                            message: Some("发送失败（上传错误）".to_string()),
                        });
                    }
                }
            }

            log::info!("Transfer complete, session_id={}", session_id);
            if let Some(peer) = remembered_peer {
                remember_peer(peer);
            }
            let _ = notify_tx.send(SendUiMessage::UpdateStatus {
                status: SendSessionStatus::Completed,
                message: Some("发送完成".to_string()),
            });
        });

        cx.spawn(async move |_this, cx| {
            while let Some(message) = notify_rx.recv().await {
                match message {
                    SendUiMessage::Notice(text) => {
                        let _ = window_handle.update(cx, |_, window, cx| {
                            let _ = home_entity.update(cx, |this, cx| {
                                this.open_simple_notice_dialog(&text, window, cx);
                            });
                        });
                    }
                    SendUiMessage::UpdateStatus { status, message } => {
                        let _ = home_entity.update(cx, |this, cx| {
                            this.send_state.session_status = status;
                            this.send_state.session_status_text = message.clone();
                            this.send_state.pending_send = !matches!(
                                status,
                                SendSessionStatus::Idle
                                    | SendSessionStatus::Completed
                                    | SendSessionStatus::Declined
                                    | SendSessionStatus::RecipientBusy
                                    | SendSessionStatus::TooManyAttempts
                                    | SendSessionStatus::CancelledByUser
                                    | SendSessionStatus::Failed
                            );

                            if status == SendSessionStatus::Completed {
                                let timestamp = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .map(|d| d.as_secs())
                                    .unwrap_or(0);
                                for file in &sent_files_for_history {
                                    let entry = HistoryEntry {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        file_name: file.name.clone(),
                                        file_size: file.size,
                                        file_path: file.path.clone(),
                                        direction: TransferDirection::Send,
                                        device_name: target_device_name.clone(),
                                        timestamp,
                                        status: TransferStatus::Completed,
                                    };
                                    let _ = history_state.update(cx, |state, _| {
                                        state.add_entry(entry);
                                    });
                                }
                            }
                        });
                    }
                    SendUiMessage::RequestPin {
                        show_invalid_pin,
                        responder,
                    } => {
                        let _ = window_handle.update(cx, |_, window, cx| {
                            let _ = home_entity.update(cx, |this, cx| {
                                this.open_send_pin_dialog(show_invalid_pin, responder, window, cx);
                            });
                        });
                    }
                }
            }
        })
        .detach();
    }

    fn render_bottom_nav(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let items: [(TabType, &'static str, &'static str); 3] = [
            (TabType::Receive, "接收", "icons/wifi.svg"),
            (TabType::Send, "发送", "icons/send-horizontal.svg"),
            (TabType::Settings, "设置", "icons/settings.svg"),
        ];

        h_flex()
            .w_full()
            .items_center()
            .children(items.iter().map(|(tab, label, icon_path)| {
                div()
                    .flex_1()
                    .child(self.render_bottom_nav_item(*tab, label, *icon_path, cx))
            }))
            .into_any_element()
    }

    fn render_bottom_nav_item(
        &mut self,
        tab: TabType,
        label: &'static str,
        icon_path: &'static str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = self.current_tab == tab;
        let tab_id = format!("tab-{:?}", tab);
        let text_color = if selected {
            cx.theme().primary
        } else {
            cx.theme().muted_foreground
        };
        let icon_el = Icon::default()
            .path(icon_path)
            .text_color(text_color)
            .with_size(gpui_component::Size::Large);

        div()
            .id(tab_id)
            .w_full()
            .h(px(56.))
            .py(px(6.))
            .flex()
            .items_center()
            .justify_center()
            .on_click(cx.listener(move |this, _event, _window, _cx| {
                this.current_tab = tab;
            }))
            .child(
                v_flex()
                    .items_center()
                    .gap(px(2.))
                    .text_color(text_color)
                    .child(icon_el)
                    .child(
                        div()
                            .when(selected, |this| this.text_base())
                            .when(!selected, |this| this.text_sm())
                            .child(label),
                    ),
            )
            .into_any_element()
    }
}

fn detect_primary_route_ipv4() -> Option<Ipv4Addr> {
    let probes = [("224.0.0.167", 53317), ("1.1.1.1", 80), ("8.8.8.8", 80)];
    for (host, port) in probes {
        let socket = match std::net::UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(_) => continue,
        };
        if socket.connect((host, port)).is_err() {
            continue;
        }
        let local = match socket.local_addr() {
            Ok(addr) => addr,
            Err(_) => continue,
        };
        if let std::net::IpAddr::V4(ip) = local.ip() {
            return Some(ip);
        }
    }
    None
}

fn detect_local_ipv4s(settings: &SettingsPageState) -> Vec<String> {
    let mut local_ips = Vec::<Ipv4Addr>::new();
    if let Ok(interfaces) = if_addrs::get_if_addrs() {
        for iface in interfaces {
            if iface.is_loopback() {
                continue;
            }
            if let if_addrs::IfAddr::V4(v4) = iface.addr {
                if v4.ip.is_link_local() {
                    continue;
                }
                local_ips.push(v4.ip);
            }
        }
    }

    let primary = detect_primary_route_ipv4();
    rank_ipv4_addresses(&mut local_ips, primary);

    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for ip in local_ips {
        let ip_text = ip.to_string();
        if !ip_matches_network_filters(&ip_text, settings) {
            continue;
        }
        if seen.insert(ip_text.clone()) {
            out.push(ip_text);
        }
    }

    if out.is_empty() {
        if let Some(ip) = primary {
            out.push(ip.to_string());
        }
    }

    out
}

fn is_network_filter_active(settings: &SettingsPageState) -> bool {
    !matches!(settings.network_filter_mode, NetworkFilterMode::All)
        && !settings.network_filters.is_empty()
}

fn ip_matches_network_filters(ip: &str, settings: &SettingsPageState) -> bool {
    if matches!(settings.network_filter_mode, NetworkFilterMode::All)
        || settings.network_filters.is_empty()
    {
        return true;
    }
    let matched = settings
        .network_filters
        .iter()
        .any(|rule| wildcard_ip_match(ip, rule));
    match settings.network_filter_mode {
        NetworkFilterMode::All => true,
        NetworkFilterMode::Whitelist => matched,
        NetworkFilterMode::Blacklist => !matched,
    }
}

fn wildcard_ip_match(ip: &str, rule: &str) -> bool {
    let value = ip.trim();
    let pattern = rule.trim();
    if value.is_empty() || pattern.is_empty() {
        return false;
    }
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return value.starts_with(prefix);
    }
    value == pattern
}

fn normalize_device_type_label(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn parse_device_type(value: &str) -> Option<DeviceType> {
    match normalize_device_type_label(value).as_str() {
        "mobile" => Some(DeviceType::Mobile),
        "desktop" => Some(DeviceType::Desktop),
        "web" => Some(DeviceType::Web),
        "headless" => Some(DeviceType::Headless),
        "server" => Some(DeviceType::Server),
        _ => Some(DeviceType::Desktop),
    }
}

fn normalized_device_model(settings: &SettingsPageState) -> Option<String> {
    let text = settings.device_model.trim();
    if text.is_empty() {
        Some("OpenHarmony".to_string())
    } else {
        Some(text.to_string())
    }
}

fn rank_ipv4_addresses(list: &mut Vec<Ipv4Addr>, primary: Option<Ipv4Addr>) {
    list.sort_by(|a, b| {
        let score = |ip: &Ipv4Addr| -> i32 {
            if Some(*ip) == primary {
                10
            } else if ip.octets()[3] == 1 {
                0
            } else {
                1
            }
        };
        score(b)
            .cmp(&score(a))
            .then_with(|| a.octets().cmp(&b.octets()))
    });
}

fn generate_random_alias() -> String {
    const ADJECTIVES: &[&str] = &[
        "Adorable",
        "Beautiful",
        "Big",
        "Bright",
        "Clean",
        "Clever",
        "Cool",
        "Cute",
        "Cunning",
        "Determined",
        "Energetic",
        "Efficient",
        "Fantastic",
        "Fast",
        "Fine",
        "Fresh",
        "Good",
        "Gorgeous",
        "Great",
        "Handsome",
        "Hot",
        "Kind",
        "Lovely",
        "Mystic",
        "Neat",
        "Nice",
        "Patient",
        "Pretty",
        "Powerful",
        "Rich",
        "Secret",
        "Smart",
        "Solid",
        "Special",
        "Strategic",
        "Strong",
        "Tidy",
        "Wise",
    ];
    const FRUITS: &[&str] = &[
        "Apple",
        "Avocado",
        "Banana",
        "Blackberry",
        "Blueberry",
        "Broccoli",
        "Carrot",
        "Cherry",
        "Coconut",
        "Grape",
        "Lemon",
        "Lettuce",
        "Mango",
        "Melon",
        "Mushroom",
        "Onion",
        "Orange",
        "Papaya",
        "Peach",
        "Pear",
        "Pineapple",
        "Potato",
        "Pumpkin",
        "Raspberry",
        "Strawberry",
        "Tomato",
    ];

    let seed = uuid::Uuid::new_v4();
    let bytes = seed.as_bytes();
    let adj_idx = (u16::from(bytes[0]) << 8 | u16::from(bytes[1])) as usize % ADJECTIVES.len();
    let fruit_idx = (u16::from(bytes[2]) << 8 | u16::from(bytes[3])) as usize % FRUITS.len();
    format!("{} {}", ADJECTIVES[adj_idx], FRUITS[fruit_idx])
}

fn ipv4_prefix(ip: &str) -> Option<String> {
    let mut parts = ip.split('.');
    let a: u8 = parts.next()?.parse().ok()?;
    let b: u8 = parts.next()?.parse().ok()?;
    let c: u8 = parts.next()?.parse().ok()?;
    Some(format!("{}.{}.{}", a, b, c))
}
