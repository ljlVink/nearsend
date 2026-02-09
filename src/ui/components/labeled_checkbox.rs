use gpui::{div, prelude::*, px, IntoElement, Window};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex, ActiveTheme as _, StyledExt as _,
};

/// Labeled checkbox matching localsend's LabeledCheckbox
#[derive(IntoElement)]
pub struct LabeledCheckbox {
    label: String,
    checked: bool,
    label_first: bool,
    on_changed: Option<std::rc::Rc<dyn Fn(bool) + 'static>>,
}

impl LabeledCheckbox {
    pub fn new(label: impl Into<String>, checked: bool) -> Self {
        Self {
            label: label.into(),
            checked,
            label_first: false,
            on_changed: None,
        }
    }

    pub fn label_first(mut self, label_first: bool) -> Self {
        self.label_first = label_first;
        self
    }

    pub fn on_changed<F>(mut self, handler: F) -> Self
    where
        F: Fn(bool) + 'static,
    {
        self.on_changed = Some(std::rc::Rc::new(handler));
        self
    }
}

impl gpui::RenderOnce for LabeledCheckbox {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let checked = self.checked;
        let on_changed = self.on_changed.clone();
        let label = self.label.clone();

        let checkbox = Button::new("labeled-checkbox")
            .with_variant(if checked {
                gpui_component::button::ButtonVariant::Primary
            } else {
                gpui_component::button::ButtonVariant::Secondary
            })
            .outline()
            .w(px(20.))
            .h(px(20.))
            .on_click(move |_event, _window, _cx| {
                if let Some(ref handler) = on_changed {
                    handler(!checked);
                }
            })
            .child(if checked {
                div().text_sm().child("✓")
            } else {
                div()
            });

        if self.label_first {
            h_flex()
                .gap(px(5.))
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().foreground)
                        .child(label),
                )
                .child(checkbox)
        } else {
            h_flex().gap(px(5.)).items_center().child(checkbox).child(
                div()
                    .text_sm()
                    .text_color(cx.theme().foreground)
                    .child(label),
            )
        }
    }
}
