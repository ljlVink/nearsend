use gpui::{div, prelude::*, px, Window};
use gpui_component::{ActiveTheme as _, StyledExt as _};

/// Device badge component matching localsend's DeviceBadge
#[derive(IntoElement)]
pub struct DeviceBadge {
    label: String,
    background_color: Option<gpui::Rgba>,
    foreground_color: Option<gpui::Rgba>,
    border_color: Option<gpui::Rgba>,
}

impl DeviceBadge {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            background_color: None,
            foreground_color: None,
            border_color: None,
        }
    }

    pub fn background_color(mut self, color: gpui::Rgba) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn foreground_color(mut self, color: gpui::Rgba) -> Self {
        self.foreground_color = Some(color);
        self
    }

    pub fn border_color(mut self, color: gpui::Rgba) -> Self {
        self.border_color = Some(color);
        self
    }
}

impl gpui::RenderOnce for DeviceBadge {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let bg_color = self
            .background_color
            .unwrap_or_else(|| cx.theme().muted.into());
        let fg_color = self
            .foreground_color
            .unwrap_or_else(|| cx.theme().foreground.into());
        let border_color = self
            .border_color
            .unwrap_or_else(|| cx.theme().border.into());

        div()
            .max_w(px(140.))
            .px(px(10.))
            .py(px(4.))
            .bg(bg_color)
            .rounded_full()
            .border_1()
            .border_color(border_color)
            .child(
                div()
                    .w_full()
                    .overflow_hidden()
                    .truncate()
                    .text_xs()
                    .font_semibold()
                    .text_color(fg_color)
                    .child(self.label),
            )
    }
}
