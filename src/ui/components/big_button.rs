use gpui::{div, prelude::*, px, IntoElement, Window};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    v_flex, ActiveTheme as _, Sizable as _, StyledExt as _,
};

/// Big button component matching localsend's BigButton design
#[derive(IntoElement)]
pub struct BigButton {
    icon: String, // Emoji or icon identifier
    label: String,
    filled: bool,
    on_tap: Option<std::rc::Rc<dyn Fn(&mut Window, &mut gpui::App) + 'static>>,
}

impl BigButton {
    pub const DESKTOP_WIDTH: f32 = 100.0;
    pub const MOBILE_WIDTH: f32 = 90.0;
    pub const HEIGHT: f32 = 65.0;

    pub fn new(icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            label: label.into(),
            filled: false,
            on_tap: None,
        }
    }

    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    pub fn on_tap<F>(mut self, handler: F) -> Self
    where
        F: Fn(&mut Window, &mut gpui::App) + 'static,
    {
        self.on_tap = Some(std::rc::Rc::new(handler));
        self
    }
}

impl gpui::RenderOnce for BigButton {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let on_tap = self.on_tap.clone();
        let icon = self.icon.clone();
        let label = self.label.clone();
        let filled = self.filled;

        // Use mobile width for now (can be made responsive later)
        let button_width = Self::MOBILE_WIDTH;

        let mut button = Button::new("big-button")
            .with_variant(if filled {
                gpui_component::button::ButtonVariant::Primary
            } else {
                gpui_component::button::ButtonVariant::Secondary
            })
            .w(px(button_width))
            .h(px(Self::HEIGHT));

        if !filled {
            button = button.outline();
        }

        button
            .on_click(move |_event, window, cx| {
                if let Some(ref handler) = on_tap {
                    handler(window, cx);
                }
            })
            .child(
                v_flex()
                    .items_center()
                    .justify_between()
                    .gap(px(4.))
                    .child(div().text_2xl().child(icon))
                    .child(div().text_sm().text_center().child(label)),
            )
    }
}
