use gpui::{div, prelude::*, px, IntoElement, Window};
use gpui_component::{ActiveTheme as _, StyledExt as _};

/// Switch component matching localsend's Switch/toggle design.
/// Parent should wrap with on_click for state updates.
#[derive(IntoElement)]
pub struct Switch {
    checked: bool,
}

impl Switch {
    pub fn new(checked: bool) -> Self {
        Self { checked }
    }
}

impl gpui::RenderOnce for Switch {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let checked = self.checked;

        let track_color = if checked {
            cx.theme().primary
        } else {
            cx.theme().muted
        };
        let thumb_color = if checked {
            cx.theme().primary_foreground
        } else {
            cx.theme().border
        };

        div()
            .id("switch")
            .w(px(44.))
            .h(px(24.))
            .rounded_full()
            .bg(track_color)
            .relative()
            .child(
                div()
                    .absolute()
                    .top(px(2.))
                    .left(if checked { px(22.) } else { px(2.) })
                    .w(px(20.))
                    .h(px(20.))
                    .rounded_full()
                    .bg(thumb_color),
            )
    }
}
