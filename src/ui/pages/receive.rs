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

/// Receive page for showing received files and history (mobile-first design)
pub struct ReceivePage {
    transfer_state: Entity<TransferState>,
}

impl ReceivePage {
    pub fn new(transfer_state: Entity<TransferState>) -> Self {
        Self { transfer_state }
    }
}

impl gpui::Render for ReceivePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let transfer_state = self.transfer_state.clone();

        v_flex()
            .w_full()
            .h_full()
            .p(spacing::MD)
            .gap(spacing::LG)
            .bg(cx.theme().background)
            .child(
                // Status section
                div()
                    .bg(cx.theme().muted)
                    .rounded_lg()
                    .p(sizing::CARD_PADDING)
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        v_flex()
                            .gap(spacing::SM)
                            .child(
                                div()
                                    .text_lg()
                                    .font_semibold()
                                    .text_color(cx.theme().foreground)
                                    .child("Receiving"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Ready to receive files from nearby devices"),
                            ),
                    ),
            )
            .child(
                // Transfer history section
                v_flex()
                    .gap(spacing::MD)
                    .w_full()
                    .flex_1()
                    .child(
                        div()
                            .text_lg()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("History"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .child(
                                // TODO: Fetch transfers from transfer_state and render TransferItem for each
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .p(spacing::LG)
                                    .text_center()
                                    .child("No transfers yet"),
                            ),
                    ),
            )
    }
}
