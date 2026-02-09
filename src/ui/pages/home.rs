use crate::state::{device_state::DeviceState, transfer_state::TransferState};
use crate::ui::pages::{send::SendPage, receive::ReceivePage};
use gpui::{
    div, prelude::*, px, Context, Entity, Window,
};
use gpui_component::{
    tab::{Tab, TabBar},
    v_flex, ActiveTheme as _, StyledExt as _,
};
use crate::ui::theme::sizing;

/// Home page with tab navigation (mobile-first design)
pub struct HomePage {
    device_state: Entity<DeviceState>,
    transfer_state: Entity<TransferState>,
    current_tab: usize, // 0 = Send, 1 = Receive
}

impl HomePage {
    pub fn new(
        device_state: Entity<DeviceState>,
        transfer_state: Entity<TransferState>,
    ) -> Self {
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
                // Header
                div()
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
                // Tab Bar (mobile-friendly)
                TabBar::new("main-tabs")
                    .segmented()
                    .w_full()
                    .selected_index(current_tab)
                    .on_click(cx.listener(|this, index, _window, _cx| {
                        this.current_tab = *index;
                    }))
                    .children([
                        Tab::new().label("Send"),
                        Tab::new().label("Receive"),
                    ]),
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
