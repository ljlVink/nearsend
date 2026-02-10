use gpui::{div, img, prelude::*, px, IntoElement, Window};
use gpui_component::{v_flex, ActiveTheme as _, StyledExt as _};

/// Logo component: shows app logo image with optional "NearSend" text.
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
        let primary = cx.theme().primary;
        let primary_fg = cx.theme().primary_foreground;
        let fallback_bg = cx.theme().muted;
        let fallback_fg = cx.theme().foreground;

        let logo_path = if size <= 128.0 {
            "img/logo-128.png"
        } else {
            "img/logo-256.png"
        };
        let logo = div()
            .w(px(size))
            .h(px(size))
            .rounded_lg()
            .overflow_hidden()
            .bg(primary)
            .child(
                img(logo_path)
                    .w(px(size))
                    .h(px(size))
                    .object_fit(gpui::ObjectFit::Cover)
                    .with_fallback(move || {
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(fallback_bg)
                            .child(
                                div()
                                    .text_2xl()
                                    .font_bold()
                                    .text_color(fallback_fg)
                                    .child("NS"),
                            )
                            .into_any_element()
                    }),
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
