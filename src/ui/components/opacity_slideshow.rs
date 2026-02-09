use gpui::{div, prelude::*, px, IntoElement, Window};
use gpui_component::{ActiveTheme as _, StyledExt as _};

/// Simple slideshow that cycles through text children.
/// For full animation support, would need timer/cx.spawn integration.
/// Currently shows first child - can be extended with index state.
#[derive(IntoElement)]
pub struct OpacitySlideshow {
    children: Vec<String>,
    /// Current index (0-based)
    index: usize,
}

impl OpacitySlideshow {
    pub fn new(children: Vec<String>) -> Self {
        Self { children, index: 0 }
    }

    pub fn index(mut self, index: usize) -> Self {
        self.index = index.min(self.children.len().saturating_sub(1));
        self
    }
}

impl gpui::RenderOnce for OpacitySlideshow {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let index = self.index.min(self.children.len().saturating_sub(1));
        let text = self.children.get(index).cloned().unwrap_or_default();

        div()
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .text_center()
            .child(text)
    }
}
