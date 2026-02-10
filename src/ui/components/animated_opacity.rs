use gpui::{div, prelude::*, AnyElement, Animation, AnimationExt as _, ElementId, IntoElement, Window};
use std::time::Duration;

#[derive(IntoElement)]
pub struct AnimatedOpacity {
    id: ElementId,
    opacity: f32,
    duration_ms: u64,
    child: AnyElement,
}

struct OpacityState {
    value: f32,
}

impl AnimatedOpacity {
    pub fn new(id: impl Into<ElementId>, opacity: f32, child: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            opacity,
            duration_ms: 200,
            child: child.into_any_element(),
        }
    }

    pub fn duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

impl gpui::RenderOnce for AnimatedOpacity {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_window, _cx| OpacityState {
            value: self.opacity,
        });
        let prev_value = state.read(cx).value;

        if (prev_value - self.opacity).abs() < f32::EPSILON {
            return div()
                .opacity(self.opacity)
                .child(self.child)
                .into_any_element();
        }

        state.update(cx, |state, _cx| {
            state.value = self.opacity;
        });

        let from = prev_value;
        let to = self.opacity;
        div()
            .child(self.child)
            .with_animation(
                self.id.clone(),
                Animation::new(Duration::from_millis(self.duration_ms)),
                move |this, delta| this.opacity(from + (to - from) * delta),
            )
            .into_any_element()
    }
}
