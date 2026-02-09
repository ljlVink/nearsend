use gpui::{AnyElement, IntoElement, Window};
use gpui_component::{ActiveTheme as _, StyledExt as _};

/// Rotating widget matching localsend's RotatingWidget
/// Note: Full rotation animation requires timer/cx.spawn integration
#[derive(IntoElement)]
pub struct RotatingWidget {
    spinning: bool,
    reverse: bool,
    duration_secs: u32,
    child: AnyElement,
}

impl RotatingWidget {
    pub fn new(child: impl IntoElement) -> Self {
        Self {
            spinning: false,
            reverse: false,
            duration_secs: 2,
            child: child.into_any_element(),
        }
    }

    pub fn spinning(mut self, spinning: bool) -> Self {
        self.spinning = spinning;
        self
    }

    pub fn reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }

    pub fn duration(mut self, duration_secs: u32) -> Self {
        self.duration_secs = duration_secs;
        self
    }
}

impl gpui::RenderOnce for RotatingWidget {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        // For now, just render the child. Full rotation animation would need timer integration
        self.child
    }
}
