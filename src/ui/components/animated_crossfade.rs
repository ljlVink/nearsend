use gpui::{div, prelude::*, AnyElement, IntoElement};

use super::animated_opacity::AnimatedOpacity;

#[derive(IntoElement)]
pub struct AnimatedCrossFade {
    id: String,
    show_second: bool,
    duration_ms: u64,
    first: AnyElement,
    second: AnyElement,
}

impl AnimatedCrossFade {
    pub fn new(id: impl Into<String>, show_second: bool) -> Self {
        Self {
            id: id.into(),
            show_second,
            duration_ms: 200,
            first: div().into_any_element(),
            second: div().into_any_element(),
        }
    }

    pub fn duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    pub fn first(mut self, child: impl IntoElement) -> Self {
        self.first = child.into_any_element();
        self
    }

    pub fn second(mut self, child: impl IntoElement) -> Self {
        self.second = child.into_any_element();
        self
    }
}

impl gpui::RenderOnce for AnimatedCrossFade {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let first_id = format!("{}-first", self.id);
        let second_id = format!("{}-second", self.id);
        let first_opacity = if self.show_second { 0.0 } else { 1.0 };
        let second_opacity = if self.show_second { 1.0 } else { 0.0 };

        div()
            .relative()
            .child(
                div().absolute().inset_0().child(
                    AnimatedOpacity::new(first_id, first_opacity, self.first)
                        .duration_ms(self.duration_ms),
                ),
            )
            .child(
                div().absolute().inset_0().child(
                    AnimatedOpacity::new(second_id, second_opacity, self.second)
                        .duration_ms(self.duration_ms),
                ),
            )
    }
}
