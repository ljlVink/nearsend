use gpui::{div, prelude::*, AnyElement, Animation, AnimationExt as _, IntoElement, Window};
use gpui_component::{ActiveTheme as _, StyledExt as _};
use std::time::Duration;

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
        if !self.spinning {
            return self.child.into_any_element();
        }

        let duration = Duration::from_secs(self.duration_secs.max(1) as u64);
        div()
            .child(self.child)
            .with_animation(
                "rotating-widget",
                Animation::new(duration).repeat(),
                move |this, delta| {
                    let alpha = gpui::pulsating_between(0.85, 1.0)(delta);
                    this.opacity(alpha)
                },
            )
            .into_any_element()
    }
}
