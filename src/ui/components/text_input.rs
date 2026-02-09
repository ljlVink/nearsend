use gpui::{div, prelude::*, px, AnyElement, IntoElement, Window};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    ActiveTheme as _, StyledExt as _,
};

/// Text input component matching localsend's TextFieldTv/TextFieldWithActions
#[derive(IntoElement)]
pub struct TextInput {
    value: String,
    placeholder: String,
    on_changed: Option<std::rc::Rc<dyn Fn(String) + 'static>>,
    actions: Vec<AnyElement>,
}

impl TextInput {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            placeholder: String::new(),
            on_changed: None,
            actions: Vec::new(),
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn on_changed<F>(mut self, handler: F) -> Self
    where
        F: Fn(String) + 'static,
    {
        self.on_changed = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}

impl gpui::RenderOnce for TextInput {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        // For mobile, show as editable text field
        // For now, show as button that opens dialog (like TextFieldWithActions)
        let value = self.value.clone();
        let _actions = self.actions;

        Button::new("text-input")
            .with_variant(gpui_component::button::ButtonVariant::Secondary)
            .outline()
            .w_full()
            .on_click(move |_event, _window, _cx| {
                // TODO: Open dialog with text field
                log::info!("Text input clicked");
            })
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().foreground)
                    .child(value),
            )
    }
}
