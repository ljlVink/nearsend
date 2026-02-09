use gpui::{div, prelude::*, px, Window};
use gpui_component::{ActiveTheme as _, StyledExt as _};

/// Device badge component matching localsend's DeviceBadge
#[derive(IntoElement)]
pub struct DeviceBadge {
    label: String,
    background_color: Option<gpui::Rgba>,
    foreground_color: Option<gpui::Rgba>,
}

impl DeviceBadge {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            background_color: None,
            foreground_color: None,
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
}

impl gpui::RenderOnce for DeviceBadge {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let bg_color = self
            .background_color
            .unwrap_or_else(|| cx.theme().muted.into());
        let fg_color = self
            .foreground_color
            .unwrap_or_else(|| cx.theme().foreground.into());

        div()
            .px(px(8.))
            .py(px(4.))
            .bg(bg_color)
            .rounded_md()
            .child(div().text_sm().text_color(fg_color).child(self.label))
    }
}
