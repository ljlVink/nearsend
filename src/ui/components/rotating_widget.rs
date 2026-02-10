use gpui::{div, prelude::*, Animation, AnimationExt as _, AnyElement, IntoElement, Window};
use std::time::Duration;

/// Wraps a child and optionally applies a continuous pulsing animation when spinning.
/// GPUI does not support CSS-like transform:rotate() on arbitrary elements,
/// so we use opacity pulsing as the visual indicator of activity.
/// For SVG icons, use Icon::with_transformation(Transformation::rotate(...)) directly.
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

    pub fn reverse(mut self, _reverse: bool) -> Self {
        self.reverse = _reverse;
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
                    let alpha = gpui::pulsating_between(0.7, 1.0)(delta);
                    this.opacity(alpha)
                },
            )
            .into_any_element()
    }
}
