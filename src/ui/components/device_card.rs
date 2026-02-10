use crate::ui::components::{device_badge::DeviceBadge, progress_bar::ProgressBar};
use crate::ui::theme::{sizing, spacing};
use gpui::{div, prelude::*, px, Window};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex, v_flex, ActiveTheme as _, Icon, Sizable as _, Size, StyledExt as _,
};
use localsend::http::state::ClientInfo;
use localsend::model::discovery::DeviceType;

/// Device card component matching localsend's DeviceListTile design
#[derive(IntoElement)]
pub struct DeviceCard {
    device: ClientInfo,
    is_favorite: bool,
    name_override: Option<String>,
    info: Option<String>,
    progress: Option<f64>,
    on_select: Option<std::rc::Rc<dyn Fn(&ClientInfo, &mut Window, &mut gpui::App) + 'static>>,
    on_favorite_tap:
        Option<std::rc::Rc<dyn Fn(&ClientInfo, &mut Window, &mut gpui::App) + 'static>>,
}

impl DeviceCard {
    pub fn new(device: ClientInfo) -> Self {
        Self {
            device,
            is_favorite: false,
            name_override: None,
            info: None,
            progress: None,
            on_select: None,
            on_favorite_tap: None,
        }
    }

    pub fn is_favorite(mut self, is_favorite: bool) -> Self {
        self.is_favorite = is_favorite;
        self
    }

    pub fn name_override(mut self, name: impl Into<String>) -> Self {
        self.name_override = Some(name.into());
        self
    }

    pub fn info(mut self, info: impl Into<String>) -> Self {
        self.info = Some(info.into());
        self
    }

    pub fn progress(mut self, progress: Option<f64>) -> Self {
        self.progress = progress;
        self
    }

    pub fn on_select<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClientInfo, &mut Window, &mut gpui::App) + 'static,
    {
        self.on_select = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn on_favorite_tap<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClientInfo, &mut Window, &mut gpui::App) + 'static,
    {
        self.on_favorite_tap = Some(std::rc::Rc::new(handler));
        self
    }
}

fn device_type_icon_path(device_type: &Option<DeviceType>) -> &'static str {
    match device_type {
        Some(DeviceType::Mobile) => "icons/smartphone.svg",
        Some(DeviceType::Desktop) => "icons/monitor.svg",
        Some(DeviceType::Web) => "icons/globe.svg",
        Some(DeviceType::Server) | Some(DeviceType::Headless) => "icons/server.svg",
        None => "icons/smartphone.svg",
    }
}

impl gpui::RenderOnce for DeviceCard {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let device = self.device.clone();
        let on_select = self.on_select.clone();
        let on_favorite_tap = self.on_favorite_tap.clone();
        let device_name = self.name_override.unwrap_or_else(|| device.alias.clone());
        let is_favorite = self.is_favorite;
        let info = self.info.clone();
        let progress = self.progress;
        let icon_path = device_type_icon_path(&device.device_type);

        let subtitle = if let Some(ref info_text) = info {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(info_text.clone())
                .into_any_element()
        } else if let Some(progress_val) = progress {
            ProgressBar::new(Some(progress_val)).into_any_element()
        } else {
            h_flex()
                .gap(px(10.))
                .flex_wrap()
                .child(
                    DeviceBadge::new("LAN")
                        .background_color(cx.theme().muted.into())
                        .foreground_color(cx.theme().foreground.into()),
                )
                .when(device.device_model.is_some(), |this| {
                    this.child(
                        DeviceBadge::new(device.device_model.as_ref().unwrap().clone())
                            .background_color(cx.theme().muted.into())
                            .foreground_color(cx.theme().foreground.into()),
                    )
                })
                .into_any_element()
        };

        div()
            .bg(cx.theme().secondary)
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p(sizing::CARD_PADDING)
            .mb(spacing::MD)
            .child(
                h_flex()
                    .items_start()
                    .gap(spacing::MD)
                    .w_full()
                    .child(
                        div()
                            .w(px(46.))
                            .h(px(46.))
                            .rounded_md()
                            .bg(cx.theme().muted)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Icon::default()
                                    .path(icon_path)
                                    .with_size(Size::Large)
                                    .text_color(cx.theme().foreground),
                            ),
                    )
                    .child(
                        v_flex()
                            .gap(px(5.))
                            .flex_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_semibold()
                                    .text_color(cx.theme().foreground)
                                    .child(device_name),
                            )
                            .child(subtitle),
                    )
                    .child(if on_favorite_tap.is_some() || on_select.is_some() {
                        Button::new("favorite")
                            .ghost()
                            .on_click(move |_event, window, cx| {
                                if let Some(ref handler) = on_favorite_tap {
                                    handler(&device, window, cx);
                                } else if let Some(ref handler) = on_select {
                                    handler(&device, window, cx);
                                }
                            })
                            .child(
                                Icon::default()
                                    .path("icons/heart.svg")
                                    .with_size(Size::Medium)
                                    .text_color(if is_favorite {
                                        cx.theme().danger
                                    } else {
                                        cx.theme().muted_foreground
                                    }),
                            )
                            .into_any_element()
                    } else {
                        Button::new("send")
                            .primary()
                            .on_click(move |_event, window, cx| {
                                if let Some(ref handler) = on_select {
                                    handler(&device, window, cx);
                                }
                            })
                            .child("发送")
                            .into_any_element()
                    }),
            )
    }
}
