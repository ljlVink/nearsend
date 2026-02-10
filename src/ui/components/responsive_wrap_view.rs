use gpui::{div, prelude::*, px, AnyElement, IntoElement, Window};
use gpui_component::{h_flex, ActiveTheme as _};

/// Responsive wrap view matching localsend's ResponsiveWrapView
#[derive(IntoElement)]
pub struct ResponsiveWrapView {
    outer_horizontal_padding: f32,
    outer_vertical_padding: f32,
    child_padding: f32,
    min_child_width: f32,
    children: Vec<AnyElement>,
}

impl ResponsiveWrapView {
    pub fn new(min_child_width: f32) -> Self {
        Self {
            outer_horizontal_padding: 15.0,
            outer_vertical_padding: 10.0,
            child_padding: 10.0,
            min_child_width,
            children: Vec::new(),
        }
    }

    pub fn outer_horizontal_padding(mut self, padding: f32) -> Self {
        self.outer_horizontal_padding = padding;
        self
    }

    pub fn outer_vertical_padding(mut self, padding: f32) -> Self {
        self.outer_vertical_padding = padding;
        self
    }

    pub fn child_padding(mut self, padding: f32) -> Self {
        self.child_padding = padding;
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl gpui::RenderOnce for ResponsiveWrapView {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        div()
            .px(px(self.outer_horizontal_padding))
            .py(px(self.outer_vertical_padding))
            .child(
                h_flex()
                    .gap(px(self.child_padding))
                    .flex_wrap()
                    .w_full()
                    .justify_start()
                    .children(
                        self.children
                            .into_iter()
                            .map(|child| div().w(px(self.min_child_width)).child(child)),
                    ),
            )
    }
}
