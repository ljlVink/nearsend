//! Selected files page: review and manage files before sending.
//! Route: /send/files

use crate::ui::pages::home::SelectedFileInfo;
use crate::ui::theme::spacing;
use gpui::{div, prelude::*, px, Context, Window};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex, v_flex, ActiveTheme as _, Icon, Sizable as _, Size, StyledExt as _,
};
use gpui_router::RouterState;

/// Selected files page state.
pub struct SelectedFilesPage {
    files: Vec<SelectedFileInfo>,
}

impl SelectedFilesPage {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn set_files(&mut self, files: Vec<SelectedFileInfo>) {
        self.files = files;
    }
}

impl gpui::Render for SelectedFilesPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let files = self.files.clone();
        let total_size: u64 = files.iter().map(|f| f.size).sum();
        let file_count = files.len();

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            // App bar
            .child(
                h_flex()
                    .w_full()
                    .h(px(56.))
                    .px(px(15.))
                    .items_center()
                    .child(
                        Button::new("files-back")
                            .ghost()
                            .child(
                                Icon::default()
                                    .path("icons/arrow-left.svg")
                                    .with_size(Size::Small),
                            )
                            .on_click(cx.listener(|_this, _event, window, cx| {
                                RouterState::global_mut(cx).location.pathname = "/".into();
                                window.refresh();
                            })),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_center()
                            .text_base()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child(format!("已选文件 ({})", file_count)),
                    )
                    .child(
                        Button::new("files-clear-all")
                            .ghost()
                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                this.files.clear();
                            }))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().danger)
                                    .child("清空"),
                            ),
                    ),
            )
            // File list
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .overflow_y_scrollbar()
                    .child(
                        v_flex()
                            .w_full()
                            .px(px(15.))
                            .gap(spacing::SM)
                            .children(files.iter().enumerate().map(|(i, file)| {
                                let file_name = file.name.clone();
                                let file_size = format_file_size(file.size);
                                div()
                                    .bg(cx.theme().secondary)
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p(px(12.))
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap(spacing::SM)
                                            .w_full()
                                            .child(
                                                div()
                                                    .w(px(36.))
                                                    .h(px(36.))
                                                    .rounded_md()
                                                    .bg(cx.theme().muted)
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(
                                                        Icon::default()
                                                            .path("icons/file.svg")
                                                            .with_size(Size::Small)
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
                                                            .text_color(cx.theme().foreground)
                                                            .child(file_name),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child(file_size),
                                                    ),
                                            )
                                            .child(
                                                Button::new(format!("delete-file-{}", i))
                                                    .ghost()
                                                    .on_click(cx.listener(move |this, _event, _window, _cx| {
                                                        if i < this.files.len() {
                                                            this.files.remove(i);
                                                        }
                                                    }))
                                                    .child(
                                                        Icon::default()
                                                            .path("icons/trash.svg")
                                                            .with_size(Size::Small)
                                                            .text_color(cx.theme().danger),
                                                    ),
                                            ),
                                    )
                            }))
                            .when(files.is_empty(), |this| {
                                this.child(
                                    div()
                                        .w_full()
                                        .py(px(40.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("暂无文件"),
                                )
                            }),
                    ),
            )
            // Bottom bar
            .child(
                div()
                    .w_full()
                    .px(px(15.))
                    .py(px(15.))
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("总计: {}", format_file_size(total_size))),
                            )
                            .child(
                                Button::new("add-more-files")
                                    .primary()
                                    .on_click(cx.listener(|_this, _event, _window, _cx| {
                                        log::info!("Add more files");
                                    }))
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap(px(6.))
                                            .child(
                                                Icon::default()
                                                    .path("icons/plus.svg")
                                                    .with_size(Size::Small),
                                            )
                                            .child("添加"),
                                    ),
                            ),
                    ),
            )
    }
}

impl Default for SelectedFilesPage {
    fn default() -> Self {
        Self::new()
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
