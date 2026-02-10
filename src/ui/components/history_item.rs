use crate::state::history_state::HistoryEntry;
use crate::state::transfer_state::TransferDirection;
use crate::ui::theme::{sizing, spacing};
use gpui::{div, prelude::*, px, Window};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex, v_flex, ActiveTheme as _, Icon, Sizable as _, Size, StyledExt as _,
};

/// History item component — renders a single transfer history entry.
#[derive(IntoElement)]
pub struct HistoryItem {
    entry: HistoryEntry,
    on_open: Option<std::rc::Rc<dyn Fn(&str, &mut Window, &mut gpui::App) + 'static>>,
    on_delete: Option<std::rc::Rc<dyn Fn(&str, &mut Window, &mut gpui::App) + 'static>>,
}

impl HistoryItem {
    pub fn new(entry: HistoryEntry) -> Self {
        Self {
            entry,
            on_open: None,
            on_delete: None,
        }
    }

    pub fn on_open<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str, &mut Window, &mut gpui::App) + 'static,
    {
        self.on_open = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn on_delete<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str, &mut Window, &mut gpui::App) + 'static,
    {
        self.on_delete = Some(std::rc::Rc::new(handler));
        self
    }
}

impl gpui::RenderOnce for HistoryItem {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let direction_icon = match self.entry.direction {
            TransferDirection::Send => "icons/upload.svg",
            TransferDirection::Receive => "icons/download.svg",
        };
        let direction_label = match self.entry.direction {
            TransferDirection::Send => "发送",
            TransferDirection::Receive => "接收",
        };

        let entry_id = self.entry.id.clone();
        let on_open = self.on_open.clone();
        let entry_id_del = self.entry.id.clone();
        let on_delete = self.on_delete.clone();

        div()
            .bg(cx.theme().secondary)
            .rounded_lg()
            .p(sizing::CARD_PADDING)
            .border_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .items_center()
                    .gap(spacing::MD)
                    .w_full()
                    .child(
                        div()
                            .w(px(40.))
                            .h(px(40.))
                            .rounded_md()
                            .bg(cx.theme().muted)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Icon::default()
                                    .path("icons/file.svg")
                                    .with_size(Size::Medium)
                                    .text_color(cx.theme().muted_foreground),
                            ),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .gap(px(2.))
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(cx.theme().foreground)
                                    .child(self.entry.file_name.clone()),
                            )
                            .child(
                                h_flex()
                                    .gap(px(8.))
                                    .items_center()
                                    .child(
                                        h_flex()
                                            .gap(px(4.))
                                            .items_center()
                                            .child(
                                                Icon::default()
                                                    .path(direction_icon)
                                                    .with_size(Size::XSmall)
                                                    .text_color(cx.theme().muted_foreground),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(direction_label),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(format_file_size(self.entry.file_size)),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(self.entry.device_name.clone()),
                                    ),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap(px(4.))
                            .when(on_open.is_some(), |this| {
                                this.child(
                                    Button::new("open-history")
                                        .ghost()
                                        .on_click(move |_event, window, cx| {
                                            if let Some(ref handler) = on_open {
                                                handler(&entry_id, window, cx);
                                            }
                                        })
                                        .child(
                                            Icon::default()
                                                .path("icons/external-link.svg")
                                                .with_size(Size::Small),
                                        ),
                                )
                            })
                            .when(on_delete.is_some(), |this| {
                                this.child(
                                    Button::new("delete-history")
                                        .ghost()
                                        .on_click(move |_event, window, cx| {
                                            if let Some(ref handler) = on_delete {
                                                handler(&entry_id_del, window, cx);
                                            }
                                        })
                                        .child(
                                            Icon::default()
                                                .path("icons/trash.svg")
                                                .with_size(Size::Small)
                                                .text_color(cx.theme().danger),
                                        ),
                                )
                            }),
                    ),
            )
    }
}

fn format_file_size(bytes: u64) -> String {
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
