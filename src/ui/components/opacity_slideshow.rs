use gpui::{div, prelude::*, Animation, AnimationExt as _, IntoElement, Window};
use gpui_component::ActiveTheme as _;
use std::time::Duration;

/// Simple slideshow that cycles through text children.
/// For full animation support, would need timer/cx.spawn integration.
/// Currently shows first child - can be extended with index state.
#[derive(IntoElement)]
pub struct OpacitySlideshow {
    children: Vec<String>,
    duration_millis: u64,
    switch_duration_millis: u64,
    running: bool,
}

impl OpacitySlideshow {
    pub fn new(children: Vec<String>) -> Self {
        Self {
            children,
            duration_millis: 6000,
            switch_duration_millis: 300,
            running: true,
        }
    }

    pub fn duration_millis(mut self, duration_millis: u64) -> Self {
        self.duration_millis = duration_millis;
        self
    }

    pub fn switch_duration_millis(mut self, switch_duration_millis: u64) -> Self {
        self.switch_duration_millis = switch_duration_millis;
        self
    }

    pub fn running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }
}

impl gpui::RenderOnce for OpacitySlideshow {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        if self.children.is_empty() {
            return div().into_any_element();
        }

        let text_style = div()
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .text_center();

        if !self.running || self.children.len() == 1 {
            return text_style
                .child(self.children[0].clone())
                .into_any_element();
        }

        let total_duration_ms = self.duration_millis * self.children.len() as u64;
        let per_slide_ms = self.duration_millis.max(1) as f32;
        let fade_ms = self.switch_duration_millis.max(1) as f32;
        let children = self.children;

        text_style
            .with_animation(
                "opacity-slideshow",
                Animation::new(Duration::from_millis(total_duration_ms)).repeat(),
                move |this, delta| {
                    let elapsed = delta * total_duration_ms as f32;
                    let mut index = (elapsed / per_slide_ms).floor() as usize;
                    if index >= children.len() {
                        index = children.len() - 1;
                    }
                    let local = elapsed - (index as f32 * per_slide_ms);
                    let mut alpha = 1.0;
                    if local < fade_ms {
                        alpha = local / fade_ms;
                    } else if local > per_slide_ms - fade_ms {
                        alpha = (per_slide_ms - local) / fade_ms;
                    }

                    this.opacity(alpha.clamp(0.0, 1.0))
                        .child(children[index].clone())
                },
            )
            .into_any_element()
    }
}
