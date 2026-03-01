//! Donate page: support information and contact details.

use crate::ui::routes;
use gpui::{div, prelude::*, px, AnyElement, Context, Entity, Window};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::{
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    h_flex, v_flex, ActiveTheme as _, Icon, Sizable as _, Size, StyledExt as _,
};

const DONATE_EMAIL: &str = "richerfu@qq.com";
const DONATE_GITHUB: &str = "https://github.com/richerfu";
const DONATE_WEBSITE: &str = "https://richerfu.win/";

/// Donate page for NearSend settings.
pub struct DonatePage {
    pub root: Option<Entity<crate::app::AppRoot>>,
}

impl DonatePage {
    pub fn new(root: Entity<crate::app::AppRoot>) -> Self {
        Self { root: Some(root) }
    }
}

fn info_card(
    id: impl Into<String>,
    title: &'static str,
    body: impl IntoElement,
    cx: &mut Context<DonatePage>,
) -> impl IntoElement {
    v_flex()
        .id(id.into())
        .w_full()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().secondary)
        .p(px(14.))
        .gap(px(8.))
        .child(
            div()
                .text_base()
                .font_semibold()
                .text_color(cx.theme().foreground)
                .child(title),
        )
        .child(body)
}

fn info_item(
    id: impl Into<String>,
    icon: &'static str,
    label: &'static str,
    value: &'static str,
    cx: &mut Context<DonatePage>,
) -> AnyElement {
    h_flex()
        .id(id.into())
        .w_full()
        .items_start()
        .gap(px(8.))
        .child(
            div()
                .h(px(24.))
                .w(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .text_color(cx.theme().muted_foreground)
                .child(Icon::default().path(icon).with_size(Size::Small)),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w(px(0.))
                .gap(px(4.))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(label),
                )
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.))
                        .text_color(cx.theme().foreground)
                        .child(value),
                ),
        )
        .into_any_element()
}

impl gpui::Render for DonatePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let intro = div()
            .text_sm()
            .line_height(px(22.))
            .text_color(cx.theme().muted_foreground)
            .child(
                "如果你觉得 NearSend 对你有帮助，欢迎通过以下方式联系我支持项目。谢谢你的关注与支持。",
            );

        let contact_info = v_flex()
            .w_full()
            .gap(px(12.))
            .child(info_item(
                "donate-email",
                "icons/inbox.svg",
                "邮箱",
                DONATE_EMAIL,
                cx,
            ))
            .child(info_item(
                "donate-github",
                "icons/github.svg",
                "GitHub",
                DONATE_GITHUB,
                cx,
            ))
            .child(info_item(
                "donate-website",
                "icons/globe.svg",
                "个人网站",
                DONATE_WEBSITE,
                cx,
            ));

        let back_button_variant = ButtonCustomVariant::new(cx)
            .hover(cx.theme().transparent)
            .active(cx.theme().transparent);

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                h_flex()
                    .w_full()
                    .h(px(56.))
                    .px(px(16.))
                    .items_center()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        h_flex()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                Button::new("donate-back")
                                    .ghost()
                                    .custom(back_button_variant)
                                    .h(px(36.))
                                    .w(px(36.))
                                    .p(px(0.))
                                    .rounded_md()
                                    .child(
                                        Icon::default()
                                            .path("icons/arrow-left.svg")
                                            .with_size(Size::Small),
                                    )
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        if let Some(root) = &this.root {
                                            let _ = root.update(cx, |root, cx| {
                                                root.go_back_or_navigate(routes::HOME, cx);
                                            });
                                        }
                                    })),
                            )
                            .child(
                                div()
                                    .text_lg()
                                    .font_semibold()
                                    .text_color(cx.theme().foreground)
                                    .child("捐赠"),
                            ),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_y_scrollbar()
                    .px(px(15.))
                    .py(px(12.))
                    .gap(px(12.))
                    .child(info_card("donate-intro", "支持项目", intro, cx))
                    .child(info_card("donate-contact", "个人信息", contact_info, cx))
                    .child(div().h(px(24.))),
            )
    }
}
