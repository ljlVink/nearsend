use crate::state::transfer_state::{TransferInfo, TransferStatus};
use crate::ui::theme::{sizing, spacing};
use gpui::{div, prelude::*, px, Context, Window};
use gpui_component::{
    h_flex, progress::Progress, v_flex, ActiveTheme as _, Sizable as _, StyledExt as _,
};

/// Transfer item component for mobile design
#[derive(IntoElement)]
pub struct TransferItem {
    transfer: TransferInfo,
}

impl TransferItem {
    pub fn new(transfer: TransferInfo) -> Self {
        Self { transfer }
    }
}

impl gpui::RenderOnce for TransferItem {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let status_text = match self.transfer.status {
            TransferStatus::Pending => "Pending",
            TransferStatus::InProgress => "In Progress",
            TransferStatus::Completed => "Completed",
            TransferStatus::Failed => "Failed",
            TransferStatus::Cancelled => "Cancelled",
        };

        div()
            .bg(cx.theme().background)
            .rounded_lg()
            .p(sizing::CARD_PADDING)
            .mb(spacing::MD)
            .border_1()
            .border_color(cx.theme().border)
            .child(
                v_flex()
                    .gap(spacing::SM)
                    .w_full()
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .w_full()
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(cx.theme().foreground)
                                    .child(self.transfer.file_name.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(status_text),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("To: {}", self.transfer.device_name)),
                    )
                    .when(self.transfer.status == TransferStatus::InProgress, |this| {
                        this.child(
                            Progress::new("transfer-progress")
                                .value((self.transfer.progress * 100.0) as f32)
                                .w_full(),
                        )
                    })
                    .when(self.transfer.status == TransferStatus::InProgress, |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!(
                                    "{:.1}% • {}/{}",
                                    self.transfer.progress * 100.0,
                                    format_bytes(self.transfer.bytes_sent),
                                    format_bytes(self.transfer.total_bytes),
                                )),
                        )
                    }),
            )
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
