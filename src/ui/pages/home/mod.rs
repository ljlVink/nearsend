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
use std::time::Duration;

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
        if let Some(ip) = detect_primary_ipv4() {
            self.receive_state.server_ips = vec![ip];
        }

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

        self.start_local_server(cx);

        let tokio_handle = self.app_state.read(cx).tokio_handle.clone();
        tokio_handle.spawn(async move {
            if let Err(err) = crate::core::multicast::start_multicast_service(
                client_info.alias,
                client_info.token,
                53317,
                client_info.device_model,
                client_info.device_type,
            )
            .await
            {
                log::warn!("multicast service failed: {}", err);
            }
        });
    }

    pub(super) fn start_local_server(&mut self, cx: &mut Context<Self>) {
        let client_info = if let Some(info) = self.app_state.read(cx).client_info.clone() {
            info
        } else {
            let token = uuid::Uuid::new_v4().to_string();
            let info = ClientInfo {
                alias: self.receive_state.server_alias.clone(),
                version: "2.1".to_string(),
                device_model: Some("OpenHarmony".to_string()),
                device_type: Some(DeviceType::Mobile),
                token,
            };
            let app_state_entity = self.app_state.clone();
            app_state_entity.update(cx, |state, _cx| {
                state.client_info = Some(info.clone());
            });
            info
        };

        let server_entity = self.app_state.read(cx).server.clone();
        let tokio_handle = self.app_state.read(cx).tokio_handle.clone();
        let cert = self.app_state.read(cx).cert.clone();
        match server_entity.update(cx, |server, _cx| {
            server.start(client_info, false, cert, &tokio_handle)
        }) {
            Ok(()) => {
                self.receive_state.server_running = true;
                self.settings_state.server_running = true;
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

    pub(super) fn start_discovery_scan(&mut self, cx: &mut Context<Self>) {
        if self.send_state.scanning {
            return;
        }
        self.send_state.has_scanned_once = true;
        self.send_state.scanning = true;
        self.send_state.nearby_devices.clear();
        self.send_state.nearby_endpoints.clear();

        let port = self.receive_state.server_port;
        let timeout_ms = self.settings_state.discovery_timeout.max(200) as u64;
        let self_fingerprint = self
            .app_state
            .read(cx)
            .client_info
            .as_ref()
            .map(|c| c.token.clone());
        let handle = self.app_state.read(cx).tokio_handle.clone();
        let join = handle.spawn(async move {
            crate::core::discovery::scan_local_network(
                port,
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
                this.send_state.nearby_devices = discovered.iter().map(|d| d.info.clone()).collect();
                this.send_state.nearby_endpoints = discovered
                    .iter()
                    .map(|d| {
                        (
                            d.info.token.clone(),
                            send_state::DeviceEndpoint {
                                ip: d.ip.clone(),
                                port: d.port,
                                https: d.https,
                            },
                        )
                    })
                    .collect();
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
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default().ok_text("确定"),
                )
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
            SendContentType::File => self.open_simple_notice_dialog("文件选择器即将接入。当前可先使用“文本”发送。", window, cx),
            SendContentType::Folder => self.open_simple_notice_dialog("文件夹选择即将接入。", window, cx),
            SendContentType::Media => self.open_simple_notice_dialog("剪贴板发送即将接入。", window, cx),
        }
    }

    pub(super) fn cycle_send_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.send_state.send_mode = match self.send_state.send_mode {
            SendMode::Single => SendMode::Multiple,
            SendMode::Multiple => SendMode::Link,
            SendMode::Link => SendMode::Single,
        };
        let mode_text = match self.send_state.send_mode {
            SendMode::Single => "单设备发送模式",
            SendMode::Multiple => "多设备发送模式（基础）",
            SendMode::Link => "链接分享模式（即将接入）",
        };
        if matches!(self.send_state.send_mode, SendMode::Link)
            && !self.ensure_has_selected_files(window, cx)
        {
            self.send_state.send_mode = SendMode::Single;
            return;
        }
        self.open_simple_notice_dialog(mode_text, window, cx);
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
                .overlay(true)
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

                fn to_v2_device_type(
                    device_type: &Option<localsend::model::discovery::DeviceType>,
                ) -> Option<V2DeviceType> {
                    match device_type {
                        Some(localsend::model::discovery::DeviceType::Mobile) => Some(V2DeviceType::Mobile),
                        Some(localsend::model::discovery::DeviceType::Desktop) => Some(V2DeviceType::Desktop),
                        Some(localsend::model::discovery::DeviceType::Web) => Some(V2DeviceType::Web),
                        Some(localsend::model::discovery::DeviceType::Headless) => Some(V2DeviceType::Headless),
                        Some(localsend::model::discovery::DeviceType::Server) => Some(V2DeviceType::Server),
                        None => None,
                    }
                }

                let files_for_v2 = files.clone();
                let mut v2_file_map = std::collections::HashMap::new();
                let mut v2_entry_map = std::collections::HashMap::new();

                for entry in files_for_v2 {
                    let file_id = uuid::Uuid::new_v4().to_string();
                    let dto = match &entry {
                        SendFileEntry::Text { content } => V2FileDto {
                            id: file_id.clone(),
                            file_name: "text.txt".to_string(),
                            size: content.as_bytes().len() as u64,
                            file_type: "text/plain".to_string(),
                            preview: Some(content.clone()),
                        },
                        SendFileEntry::File {
                            name, size, file_type, ..
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
                    let register_url = format!("{}://{}:{}/api/localsend/v2/register", scheme, ip, port);
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
                            }
                        }
                        Err(e) => {
                            log::warn!("v2 register failed via {} for {}:{}: {}", scheme, ip, port, e);
                        }
                    }

                    let prepare_url = format!("{}://{}:{}/api/localsend/v2/prepare-upload", scheme, ip, port);
                    match v2_http_client
                        .post(&prepare_url)
                        .json(&v2_payload)
                        .send()
                        .await
                    {
                        Ok(res) => {
                            if !res.status().is_success() {
                                let msg = format!("status={}", res.status());
                                log::warn!(
                                    "v2 prepare-upload failed via {} for {}:{}: {}",
                                    scheme,
                                    ip,
                                    port,
                                    msg
                                );
                                v2_last_err = Some(msg);
                                continue;
                            }
                            match res.json::<V2PrepareUploadResponseDto>().await {
                                Ok(parsed) => {
                                    v2_selected_scheme = Some(scheme);
                                    v2_response = Some(parsed);
                                    break;
                                }
                                Err(e) => {
                                    let msg = format!("decode prepare-upload response failed: {}", e);
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
                        }
                    }
                }

                if let (Some(scheme), Some(v2_response)) = (v2_selected_scheme, v2_response) {
                    log::info!(
                        "v2 prepare-upload succeeded for {}:{} over {}, session={}",
                        ip,
                        port,
                        scheme,
                        v2_response.session_id
                    );

                    for (file_id, token) in &v2_response.files {
                        if let Some(entry) = v2_entry_map.remove(file_id) {
                            let (body, content_type) = match entry {
                                SendFileEntry::Text { content } => {
                                    (content.into_bytes(), "text/plain".to_string())
                                }
                                SendFileEntry::File { path, file_type, .. } => {
                                    match tokio::fs::read(&path).await {
                                        Ok(data) => (data, file_type),
                                        Err(e) => {
                                            log::error!("Failed to read file {:?}: {}", path, e);
                                            continue;
                                        }
                                    }
                                }
                            };

                            let upload_url = format!("{}://{}:{}/api/localsend/v2/upload", scheme, ip, port);
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
                                }
                                Err(e) => {
                                    log::error!("v2 upload failed for file_id={}: {}", file_id, e);
                                }
                            }
                        }
                    }

                    log::info!(
                        "v2 transfer complete, session_id={}",
                        v2_response.session_id
                    );
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

                // Target client protocol can be HTTP or HTTPS.
                // Try HTTP first for backward compatibility, then HTTPS fallback.
                let protocols = [ProtocolType::Http, ProtocolType::Https];
                let mut selected_protocol: Option<ProtocolType> = None;
                let mut response: Option<localsend::http::dto::PrepareUploadResponseDto> = None;
                let mut last_err: Option<String> = None;

                for protocol in protocols {
                    log::info!(
                        "Trying protocol={} for {}:{}",
                        protocol.as_str(),
                        ip,
                        port
                    );

                    let public_key = match client.register(&protocol, &ip, port, payload.info.clone()).await {
                        Ok(register_result) => register_result.public_key,
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

                    match client
                        .prepare_upload(&protocol, &ip, port, public_key, payload.clone())
                        .await
                    {
                        Ok(r) => {
                            selected_protocol = Some(protocol);
                            response = Some(r);
                            break;
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
                        }
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
                        return;
                    }
                };

                let session_id = response.session_id.clone();
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

fn detect_primary_ipv4() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local = socket.local_addr().ok()?;
    match local.ip() {
        std::net::IpAddr::V4(ip) => Some(ip.to_string()),
        _ => None,
    }
}
