use gpui::{div, prelude::*, px, ElementId, IntoElement, Window};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    ActiveTheme as _, StyledExt as _,
};
use std::rc::Rc;

/// Custom icon button matching localsend's CustomIconButton
/// This is a simple wrapper that renders an icon button
#[derive(IntoElement)]
pub struct CustomIconButton {
    id: ElementId,
    child: String, // Emoji or icon identifier
    on_click: Option<Rc<dyn Fn(&mut Window, &mut gpui::App) + 'static>>,
}

impl CustomIconButton {
    pub fn new(id: impl Into<ElementId>, child: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            child: child.into(),
            on_click: None,
        }
    }

    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(&mut Window, &mut gpui::App) + 'static,
    {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl gpui::RenderOnce for CustomIconButton {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        let on_click = self.on_click.clone();

        Button::new(self.id)
            .ghost()
            .rounded_full()
            .p(px(8.))
            .when_some(on_click, |this, handler| {
                this.on_click(move |_event, window, cx| {
                    handler(window, cx);
                })
            })
            .child(div().text_lg().child(self.child))
    }
}
