use crate::state::transfer_state::TransferState;
use crate::ui::components::transfer_item::TransferItem;
use gpui::{
    div, prelude::*, px, Context, Entity, Window,
};
use gpui_component::{
    scroll::ScrollableElement,
    v_flex, ActiveTheme as _, StyledExt as _,
};
use crate::ui::theme::{spacing, sizing};

/// Progress page for showing active transfers (mobile-first design)
pub struct ProgressPage {
    transfer_state: Entity<TransferState>,
}

impl ProgressPage {
    pub fn new(transfer_state: Entity<TransferState>) -> Self {
        Self { transfer_state }
    }
}

impl gpui::Render for ProgressPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let transfer_state = self.transfer_state.clone();

        v_flex()
            .flex_1()
            .p(spacing::MD)
            .gap(spacing::MD)
            .bg(cx.theme().background)
            .child(
                div()
                    .text_lg()
                    .font_semibold()
                    .text_color(cx.theme().foreground)
                    .child("Active Transfers"),
            )
            .child(
                div()
                    .flex_1()
                    .child(
                        // TODO: Fetch active transfers from transfer_state and render TransferItem for each
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .p(spacing::LG)
                            .text_center()
                            .child("No active transfers"),
                    ),
            )
    }
}
