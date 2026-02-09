use gpui::{div, prelude::*, px, Context, IntoElement, Window};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    ActiveTheme as _, StyledExt as _,
};

/// Custom icon button matching localsend's CustomIconButton
/// This is a simple wrapper that renders an icon button
#[derive(IntoElement)]
pub struct CustomIconButton {
    child: String, // Emoji or icon identifier
}

impl CustomIconButton {
    pub fn new(child: impl Into<String>) -> Self {
        Self {
            child: child.into(),
        }
    }
}

impl gpui::RenderOnce for CustomIconButton {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        Button::new("icon-button")
            .ghost()
            .rounded_full()
            .p(px(8.))
            .child(div().text_lg().child(self.child))
    }
}
