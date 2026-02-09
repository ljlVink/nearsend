use crate::core::{discovery::DiscoveryService, server::ServerManager, transfer::TransferService};
use crate::state::{app_state::AppState, device_state::DeviceState, transfer_state::TransferState};
use gpui::{
    div, prelude::*, AnyElement, AppContext, Context, Entity, Window, AsyncApp, IntoElement, ParentElement,
};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    tab::{Tab, TabBar},
    v_flex, h_flex, Root, StyledExt as _, ActiveTheme as _, Sizable as _,
};
use localsend::http::state::ClientInfo;
use std::path::PathBuf;

/// Main application using new modular structure
pub struct NearSendAppNew {
    app_state: Entity<AppState>,
    device_state: Entity<DeviceState>,
    transfer_state: Entity<TransferState>,
    current_tab: TabType,
    services_started: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TabType {
    Send,
    Receive,
}

impl NearSendAppNew {
    /// Create a new NearSendAppNew instance
    pub fn new(
        app_state: Entity<AppState>,
        device_state: Entity<DeviceState>,
        transfer_state: Entity<TransferState>,
    ) -> Self {
        Self {
            app_state,
            device_state,
            transfer_state,
            current_tab: TabType::Send,
            services_started: false,
        }
    }
}

fn get_device_model() -> String {
    #[cfg(target_os = "macos")]
    {
        return "macOS".to_string();
    }
    #[cfg(target_os = "windows")]
    {
        return "Windows".to_string();
    }
    #[cfg(target_os = "linux")]
    {
        return "Linux".to_string();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "Unknown".to_string()
    }
}

impl gpui::Render for NearSendAppNew {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        // Start services only once using GPUI async context (OpenHarmony compatible)
        if !self.services_started {
            self.services_started = true;
            
            // Start discovery service
            // TODO: Fix spawn API - services will be started later
            // cx.spawn(|_entity, _cx: &mut AsyncApp| async move {
            //     log::info!("Starting discovery service via GPUI async context");
            // });

            // Start server
            // TODO: Fix spawn API - services will be started later
            // let client_info = ClientInfo {
            //     alias: "NearSend".to_string(),
            //     version: "2.0".to_string(),
            //     device_model: Some(get_device_model()),
            //     device_type: Some(localsend::model::discovery::DeviceType::Desktop),
            //     token: uuid::Uuid::new_v4().to_string(),
            // };
            // cx.spawn(move |_entity, _cx: &mut AsyncApp| async move {
            //     log::info!("Starting server via GPUI async context");
            // });
        }

        // Main UI container with full flex layout
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                // Header
                div()
                    .w_full()
                    .bg(cx.theme().background)
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .p_4()
                    .child(
                        div()
                            .text_xl()
                            .font_bold()
                            .text_color(cx.theme().foreground)
                            .child("NearSend"),
                    ),
            )
            .child(
                // Tab Bar
                TabBar::new("main-tabs")
                    .segmented()
                    .w_full()
                    .selected_index(if self.current_tab == TabType::Send { 0 } else { 1 })
                    .on_click(cx.listener(|this, index, _window, _cx| {
                        this.current_tab = if *index == 0 { TabType::Send } else { TabType::Receive };
                    }))
                    .children([
                        Tab::new().label("Send"),
                        Tab::new().label("Receive"),
                    ]),
            )
            .child(
                // Content area
                v_flex()
                    .flex_1()
                    .w_full()
                    .child(match self.current_tab {
                        TabType::Send => self.render_send_page(window, cx),
                        TabType::Receive => self.render_receive_page(window, cx),
                    }),
            )
            .children(sheet_layer)
            .children(dialog_layer)
            .children(notification_layer)
    }
}

impl NearSendAppNew {
    /// Render the Send page
    fn render_send_page(&self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .size_full()
            .p_4()
            .gap_4()
            .child(
                // File selection section
                v_flex()
                    .gap_3()
                    .w_full()
                    .child(
                        div()
                            .text_lg()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("Select Files"),
                    )
                    .child(
                        Button::new("choose-files")
                            .primary()
                            .w_full()
                            .h_12()
                            .on_click(cx.listener(|_this, _event, _window, _cx| {
                                log::info!("File picker clicked");
                            }))
                            .child("Choose Files"),
                    ),
            )
            .child(
                // Device list section
                v_flex()
                    .gap_3()
                    .w_full()
                    .flex_1()
                    .child(
                        div()
                            .text_lg()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("Nearby Devices"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .p_6()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .text_center()
                                    .child("Scanning for devices..."),
                            ),
                    ),
            )
            .into_any_element()
    }

    /// Render the Receive page
    fn render_receive_page(&self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .size_full()
            .p_4()
            .gap_4()
            .child(
                // Status section
                v_flex()
                    .gap_3()
                    .w_full()
                    .child(
                        div()
                            .text_lg()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("Receive Files"),
                    )
                    .child(
                        div()
                            .w_full()
                            .p_6()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .bg(cx.theme().muted)
                            .child(
                                v_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_base()
                                            .font_medium()
                                            .text_color(cx.theme().foreground)
                                            .child("Ready to Receive"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .text_center()
                                            .child("Other devices can now send files to this device"),
                                    ),
                            ),
                    ),
            )
            .child(
                // Transfer list section
                v_flex()
                    .gap_3()
                    .w_full()
                    .flex_1()
                    .child(
                        div()
                            .text_lg()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("Recent Transfers"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .p_6()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .text_center()
                                    .child("No transfers yet"),
                            ),
                    ),
            )
            .into_any_element()
    }
}
