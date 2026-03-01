use gpui::{div, prelude::*, px, IntoElement, Window};
use gpui_component::ActiveTheme as _;

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
            cx.theme().muted_foreground
        };

        div()
            .id("switch")
            .w(px(50.))
            .h(px(28.))
            .rounded_full()
            .bg(track_color)
            .border_2()
            .border_color(if checked {
                cx.theme().primary
            } else {
                cx.theme().muted_foreground
            })
            .relative()
            .child(
                div()
                    .absolute()
                    .top(px(1.))
                    .left(if checked { px(23.) } else { px(1.) })
                    .w(px(22.))
                    .h(px(22.))
                    .rounded_full()
                    .bg(thumb_color),
            )
    }
}
