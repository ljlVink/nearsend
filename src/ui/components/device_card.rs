use gpui::{
    div, prelude::*, px, Context, SharedString, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    v_flex, h_flex, ActiveTheme as _, StyledExt as _, Sizable as _,
};
use localsend::http::state::ClientInfo;
use crate::ui::theme::{spacing, sizing};

/// Device card component for mobile design
pub struct DeviceCard {
    device: ClientInfo,
    on_select: Option<std::rc::Rc<dyn Fn(&ClientInfo, &mut Window, &mut gpui::App) + 'static>>,
}

impl DeviceCard {
    pub fn new(device: ClientInfo) -> Self {
        Self {
            device,
            on_select: None,
        }
    }

    pub fn on_select<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClientInfo, &mut Window, &mut gpui::App) + 'static,
    {
        self.on_select = Some(std::rc::Rc::new(handler));
        self
    }
}

impl gpui::RenderOnce for DeviceCard {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let device = self.device.clone();
        let on_select = self.on_select.clone();

        div()
            .bg(cx.theme().background)
            .rounded_lg()
            .p(sizing::CARD_PADDING)
            .mb(spacing::MD)
            .border_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .child(
                        v_flex()
                            .gap_1()
                            .flex_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_semibold()
                                    .text_color(cx.theme().foreground)
                                    .child(device.alias.clone()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(
                                        format!(
                                            "{}{}",
                                            device.device_model.as_deref().unwrap_or("Unknown"),
                                            device.device_type.as_ref().map(|t| format!(" • {:?}", t)).unwrap_or_default()
                                        )
                                    ),
                            ),
                    )
                    .child(
                        Button::new("Send")
                            .primary()
                            .on_click(move |_event, window, cx| {
                                if let Some(ref handler) = on_select {
                                    handler(&device, window, cx);
                                }
                            }),
                    ),
            )
    }
}
