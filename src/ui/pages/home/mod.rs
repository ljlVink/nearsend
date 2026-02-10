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
use gpui_component::{h_flex, v_flex, ActiveTheme as _, Icon, Sizable as _, StyledExt as _};

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
        }
    }
}

impl gpui::Render for HomePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.services_started {
            self.services_started = true;
            // TODO: Start discovery and server services
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
