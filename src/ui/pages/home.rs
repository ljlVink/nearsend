use crate::state::{device_state::DeviceState, transfer_state::TransferState};
use crate::ui::pages::{receive::ReceivePage, send::SendPage};
use gpui::{div, prelude::*, px, Context, Entity, Window};
use gpui_component::{h_flex, v_flex, ActiveTheme as _, StyledExt as _};

/// Home page with tab navigation (mobile-first design)
pub struct HomePage {
    device_state: Entity<DeviceState>,
    transfer_state: Entity<TransferState>,
    current_tab: usize, // 0 = Send, 1 = Receive
}

impl HomePage {
    pub fn new(device_state: Entity<DeviceState>, transfer_state: Entity<TransferState>) -> Self {
        Self {
            device_state,
            transfer_state,
            current_tab: 0,
        }
    }
}

impl gpui::Render for HomePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let device_state = self.device_state.clone();
        let transfer_state = self.transfer_state.clone();
        let current_tab = self.current_tab;

        v_flex()
            .flex_1()
            .bg(cx.theme().background)
            .child(
                // Header (no divider with tab row below)
                div()
                    .bg(cx.theme().background)
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
                // Minimal tab row: no divider with content, selected = theme color (no bg), unselected = gray
                h_flex()
                    .w_full()
                    .gap(px(16.))
                    .px(px(12.))
                    .py(px(8.))
                    .child(
                        div()
                            .id("tab-send")
                            .text_sm()
                            .text_color(if current_tab == 0 {
                                cx.theme().primary
                            } else {
                                cx.theme().muted_foreground
                            })
                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                this.current_tab = 0;
                            }))
                            .child("Send"),
                    )
                    .child(
                        div()
                            .id("tab-receive")
                            .text_sm()
                            .text_color(if current_tab == 1 {
                                cx.theme().primary
                            } else {
                                cx.theme().muted_foreground
                            })
                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                this.current_tab = 1;
                            }))
                            .child("Receive"),
                    ),
            )
            .child(
                // Content area
                div()
                    .flex_1()
                    .when(current_tab == 0, |this| {
                        // TODO: Render SendPage - need to create view properly
                        this.child(div().child("Send Page"))
                    })
                    .when(current_tab == 1, |this| {
                        // TODO: Render ReceivePage - need to create view properly
                        this.child(div().child("Receive Page"))
                    }),
            )
    }
}
