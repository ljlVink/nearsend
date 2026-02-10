//! Home page: three tabs (Receive, Send, Settings) with bottom navigation.
//! Uses gpui-router; history is a separate route (see history page).

mod receive_state;
mod receive_tab;
mod send_state;
mod send_tab;
mod settings_state;
mod settings_tab;

pub use receive_state::{IncomingTransferRequest, QuickSaveMode, ReceivePageState};
pub use send_state::{SelectedFileInfo, SendContentType, SendMode, SendPageState};
pub use settings_state::{ColorMode, SettingsPageState, ThemeMode};

use crate::state::{app_state::AppState, device_state::DeviceState, transfer_state::TransferState};
use gpui::{div, prelude::*, px, AnyElement, Context, Entity, IntoElement, Window};
use gpui_component::input::{Input, InputState};
use gpui_component::select::{Select, SelectEvent, SelectState};
use gpui_component::{
    h_flex, v_flex, ActiveTheme as _, Icon, IndexPath, Sizable as _, StyledExt as _, WindowExt as _,
};
use localsend::http::state::ClientInfo;
use localsend::model::discovery::DeviceType;

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
    pub(super) send_port_input_state: Option<Entity<InputState>>,
}

impl HomePage {
    pub fn new(
        app_state: Entity<AppState>,
        device_state: Entity<DeviceState>,
        transfer_state: Entity<TransferState>,
    ) -> Self {
        Self {
            app_state,
            device_state,
            transfer_state,
            current_tab: TabType::Receive,
            services_started: false,
            receive_state: ReceivePageState::default(),
            send_state: SendPageState::default(),
            settings_state: SettingsPageState::default(),
            theme_select: None,
            color_select: None,
            language_select: None,
            text_input_state: None,
            send_ip_input_state: None,
            send_port_input_state: None,
        }
    }
}

impl gpui::Render for HomePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
    /// Start the HTTP server and discovery service.
    fn start_services(&mut self, cx: &mut Context<Self>) {
        let alias = self.receive_state.server_alias.clone();
        let token = uuid::Uuid::new_v4().to_string();
        let client_info = ClientInfo {
            alias: alias.clone(),
            version: "2.1".to_string(),
            device_model: Some("OpenHarmony".to_string()),
            device_type: Some(DeviceType::Mobile),
            token: token.clone(),
        };

        // Store client_info in AppState
        let app_state_entity = self.app_state.clone();
        app_state_entity.update(cx, |state, _cx| {
            state.client_info = Some(client_info.clone());
        });

        // Start HTTP server
        let server_entity = self.app_state.read(cx).server.clone();
        let server_client_info = client_info.clone();
        let tokio_handle = self.app_state.read(cx).tokio_handle.clone();
        match server_entity
            .update(cx, |server, _cx| server.start(server_client_info, false, &tokio_handle))
        {
            Ok(()) => {
                self.receive_state.server_running = true;
                self.settings_state.server_running = true;
                log::info!("Server started successfully");
            }
            Err(e) => {
                log::error!("Failed to start server: {}", e);
                self.receive_state.server_running = false;
            }
        }

        // Initialize transfer client with cert if available
        let cert = self.app_state.read(cx).cert.clone();
        if let Some(cert) = cert {
            let transfer_entity = self.app_state.read(cx).transfer.clone();
            transfer_entity.update(cx, |transfer, _cx| {
                transfer.init_sync(&cert.private_key_pem, &cert.cert_pem);
            });
        }

        // Start discovery (currently a stub)
        let discovery_entity = self.app_state.read(cx).discovery.clone();
        discovery_entity.update(cx, |discovery, _cx| {
            discovery.start_sync();
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
                .multi_line(true)
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
                .w(px(360.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).h(px(200.)).appearance(true)),
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
                        let text_bytes = text.as_bytes().len() as u64;
                        home_for_ok.update(cx, |this, _cx| {
                            this.send_state
                                .selected_files
                                .push(send_state::SelectedFileInfo {
                                    path: std::path::PathBuf::from("text.txt"),
                                    name: "text.txt".to_string(),
                                    size: text_bytes,
                                    file_type: "text/plain".to_string(),
                                    text_content: Some(text.clone()),
                                });
                            this.send_state.selected_files_total_size += text_bytes;
                        });
                    }
                    true
                })
        });
    }

    /// Opens a dialog to enter IP address and port for manual send.
    pub(super) fn open_send_to_address_dialog(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let ip_input_state = cx.new(|cx| InputState::new(window, cx).placeholder("输入IP地址"));
        let port_input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("端口")
                .default_value("53317")
        });
        self.send_ip_input_state = Some(ip_input_state.clone());
        self.send_port_input_state = Some(port_input_state.clone());

        let home_entity = cx.entity();

        window.open_dialog(cx, move |dialog, _window, _cx| {
            let ip_for_ok = ip_input_state.clone();
            let port_for_ok = port_input_state.clone();
            let home_for_ok = home_entity.clone();

            dialog
                .title("发送到设备")
                .w(px(360.))
                .child(
                    v_flex()
                        .w_full()
                        .gap(px(10.))
                        .child(Input::new(&ip_input_state).appearance(true))
                        .child(Input::new(&port_input_state).appearance(true)),
                )
                .confirm()
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("发送")
                        .cancel_text("取消"),
                )
                .on_ok(move |_event, _window, cx| {
                    let ip = ip_for_ok.read(cx).value().to_string();
                    let port_str = port_for_ok.read(cx).value().to_string();
                    let port: u16 = port_str.parse().unwrap_or(53317);

                    if !ip.is_empty() {
                        home_for_ok.update(cx, |this, cx| {
                            this.execute_send(ip, port, cx);
                        });
                    }
                    true
                })
        });
    }

    /// Execute the send flow: build entries from selected_files, spawn thread with tokio runtime.
    pub(super) fn execute_send(&mut self, ip: String, port: u16, cx: &mut Context<Self>) {
        use crate::core::transfer::SendFileEntry;
        use localsend::http::dto::{ProtocolType, RegisterDto};

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

        if files.is_empty() {
            log::warn!("No files selected to send");
            return;
        }

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

        log::info!(
            "Starting send to {}:{} with {} files",
            ip,
            port,
            files.len()
        );

        // Spawn on the shared tokio runtime
        tokio_handle.spawn(async move {
                // Create a fresh LsHttpClient for this transfer
                let cert = match cert {
                    Some(c) => c,
                    None => {
                        log::error!("No TLS cert available, cannot create transfer client");
                        return;
                    }
                };
                let client =
                    match localsend::http::client::LsHttpClient::try_new(
                        &cert.private_key_pem,
                        &cert.cert_pem,
                    ) {
                        Ok(c) => c,
                        Err(e) => {
                            log::error!("Failed to create transfer client: {}", e);
                            return;
                        }
                    };

                let protocol = ProtocolType::Http;

                // Build FileDto map
                let mut file_map = std::collections::HashMap::new();
                let mut entry_map = std::collections::HashMap::new();
                for entry in files {
                    let file_id = uuid::Uuid::new_v4().to_string();
                    let dto = match &entry {
                        SendFileEntry::Text { content } => localsend::model::transfer::FileDto {
                            id: file_id.clone(),
                            file_name: "text.txt".to_string(),
                            size: content.as_bytes().len() as u64,
                            file_type: "text/plain".to_string(),
                            sha256: None,
                            preview: None,
                            metadata: None,
                        },
                        SendFileEntry::File {
                            name, size, file_type, ..
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

                log::info!("Sending prepare-upload to {}:{}", ip, port);
                let response = match client
                    .prepare_upload(&protocol, &ip, port, None, payload)
                    .await
                {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("prepare-upload failed: {}", e);
                        return;
                    }
                };

                let session_id = response.session_id.clone();
                log::info!(
                    "Got session_id: {}, accepted files: {}",
                    session_id,
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
                        }
                    }
                }

                log::info!("Transfer complete, session_id={}", session_id);
            });
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
