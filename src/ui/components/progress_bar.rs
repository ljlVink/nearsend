use gpui::{div, prelude::*, px, relative, IntoElement, Window};
use gpui_component::{h_flex, ActiveTheme as _, StyledExt as _};

/// Progress bar component matching localsend's CustomProgressBar
#[derive(IntoElement)]
pub struct ProgressBar {
    progress: Option<f64>, // 0.0 to 1.0
    height: f32,
}

impl ProgressBar {
    pub fn new(progress: Option<f64>) -> Self {
        Self {
            progress,
            height: 10.0,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl gpui::RenderOnce for ProgressBar {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let progress = self.progress.unwrap_or(0.0).clamp(0.0, 1.0);

        div()
            .w_full()
            .h(px(self.height))
            .rounded_md()
            .overflow_hidden()
            .child(
                h_flex()
                    .w_full()
                    .h_full()
                    .child(
                        div()
                            .h_full()
                            .w(relative(progress as f32))
                            .bg(cx.theme().primary),
                    )
                    .child(div().h_full().flex_1().bg(cx.theme().muted)),
            )
    }
}
