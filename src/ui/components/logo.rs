use gpui::{div, prelude::*, px, IntoElement, Window};
use gpui_component::{v_flex, ActiveTheme as _, StyledExt as _};

/// Logo component matching localsend's LocalSendLogo
/// Uses primary color styled circle (image loading requires platform-specific setup)
#[derive(IntoElement)]
pub struct Logo {
    with_text: bool,
    size: f32,
}

impl Logo {
    pub fn new() -> Self {
        Self {
            with_text: false,
            size: 200.0,
        }
    }

    pub fn with_text(mut self, with_text: bool) -> Self {
        self.with_text = with_text;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl Default for Logo {
    fn default() -> Self {
        Self::new()
    }
}

impl gpui::RenderOnce for Logo {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let size = self.size;
        let with_text = self.with_text;

        let logo = div()
            .w(px(size))
            .h(px(size))
            .rounded_lg()
            .bg(cx.theme().primary)
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_2xl()
                    .font_bold()
                    .text_color(cx.theme().primary_foreground)
                    .child("NS"),
            );

        if with_text {
            v_flex().gap(px(8.)).items_center().child(logo).child(
                div()
                    .text_3xl()
                    .font_bold()
                    .text_color(cx.theme().foreground)
                    .child("NearSend"),
            )
        } else {
            logo
        }
    }
}
