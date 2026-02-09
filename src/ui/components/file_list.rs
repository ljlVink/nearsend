use gpui::{
    div, prelude::*, px, Context, SharedString, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    v_flex, h_flex, ActiveTheme as _, StyledExt as _,
};
use std::path::PathBuf;
use crate::ui::theme::{spacing, sizing};

/// File list component for mobile design
pub struct FileList {
    files: Vec<PathBuf>,
    on_remove: Option<std::rc::Rc<dyn Fn(usize, &mut Window, &mut gpui::App) + 'static>>,
}

impl FileList {
    pub fn new(files: Vec<PathBuf>) -> Self {
        Self {
            files,
            on_remove: None,
        }
    }

    pub fn on_remove<F>(mut self, handler: F) -> Self
    where
        F: Fn(usize, &mut Window, &mut gpui::App) + 'static,
    {
        self.on_remove = Some(std::rc::Rc::new(handler));
        self
    }
}

impl gpui::RenderOnce for FileList {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        if self.files.is_empty() {
            return div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .p(spacing::LG)
                .text_center()
                .child("No files selected");
        }

        v_flex()
            .gap(spacing::SM)
            .children(self.files.iter().enumerate().map(|(idx, path)| {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                let on_remove = self.on_remove.clone();
                let idx = idx;

                div()
                    .bg(cx.theme().background)
                    .rounded_lg()
                    .p(spacing::MD)
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .w_full()
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .text_color(cx.theme().foreground)
                                    .flex_1()
                                    .child(file_name),
                            )
                            .child(
                                Button::new("Remove")
                                    .ghost()
                                    .on_click(move |_event, window, cx| {
                                        if let Some(ref handler) = on_remove {
                                            handler(idx, window, cx);
                                        }
                                    }),
                            ),
                    )
            }))
    }
}
