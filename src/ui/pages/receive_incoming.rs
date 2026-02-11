use crate::state::receive_inbox_state::ReceiveInboxState;
use gpui::{div, hsla, prelude::*, px, ClipboardItem, Context, Entity, Window};
use gpui_component::{
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    h_flex, v_flex, ActiveTheme as _, Icon, Sizable as _, Size,
};
use gpui_router::RouterState;

pub struct ReceiveIncomingPage {
    inbox_state: Entity<ReceiveInboxState>,
}

impl ReceiveIncomingPage {
    pub fn new(inbox_state: Entity<ReceiveInboxState>) -> Self {
        Self { inbox_state }
    }
}

impl gpui::Render for ReceiveIncomingPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let session = self.inbox_state.read(cx).active.clone();
        let sender_alias = session
            .as_ref()
            .map(|s| s.sender_alias.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "NearSend".to_string());
        let sender_model = session
            .as_ref()
            .and_then(|s| s.sender_device_model.clone())
            .unwrap_or_else(|| "OpenHarmony".to_string());
        let sender_tag = session
            .as_ref()
            .map(|s| format!("#{}", visual_tag(&s.sender_fingerprint)))
            .unwrap_or_else(|| "#--".to_string());

        let message_content = session.as_ref().and_then(|s| {
            if !s.is_message_only {
                return None;
            }
            s.items
                .iter()
                .find_map(|item| item.text_content.clone())
                .or_else(|| s.items.first().map(|x| x.file_name.clone()))
        });
        let file_count = session.as_ref().map(|s| s.items.len()).unwrap_or(0);
        let subtitle = if message_content.is_some() {
            "发送给你了一条消息：".to_string()
        } else if file_count > 0 {
            format!("发送给你 {} 个文件", file_count)
        } else {
            "等待接收内容".to_string()
        };
        let show_cancelled = session.as_ref().map(|s| s.cancelled).unwrap_or(false);
        let close_button_variant = ButtonCustomVariant::new(cx)
            .color(cx.theme().danger.opacity(0.92))
            .foreground(hsla(0.0, 0.0, 1.0, 1.0))
            .hover(cx.theme().danger.opacity(0.84))
            .active(cx.theme().danger.opacity(0.76));

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(
                        v_flex()
                            .w_full()
                            .max_w(px(920.))
                            .px(px(20.))
                            .py(px(24.))
                            .items_center()
                            .gap(px(12.))
                            .child(
                                div()
                                    .text_3xl()
                                    .font_weight(gpui::FontWeight::BLACK)
                                    .text_color(cx.theme().foreground)
                                    .child(sender_alias),
                            )
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap(px(10.))
                                    .child(
                                        div()
                                            .px(px(10.))
                                            .py(px(4.))
                                            .rounded_md()
                                            .bg(cx.theme().primary.opacity(0.2))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::BLACK)
                                                    .text_color(cx.theme().primary)
                                                    .child(sender_tag),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .px(px(10.))
                                            .py(px(4.))
                                            .rounded_md()
                                            .bg(cx.theme().primary.opacity(0.2))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::BLACK)
                                                    .text_color(cx.theme().primary)
                                                    .child(sender_model),
                                            ),
                                    ),
                            )
                            .child(div().h(px(18.)))
                            .child(
                                div()
                                    .text_2xl()
                                    .text_color(cx.theme().foreground)
                                    .child(subtitle),
                            )
                            .when_some(message_content.clone(), |this, content| {
                                this.child(
                                    div()
                                        .w_full()
                                        .max_w(px(760.))
                                        .mt(px(8.))
                                        .min_h(px(140.))
                                        .rounded_lg()
                                        .border_1()
                                        .border_color(cx.theme().border.opacity(0.7))
                                        .bg(cx.theme().secondary)
                                        .shadow_sm()
                                        .p(px(14.))
                                        .child(
                                            div()
                                                .text_lg()
                                                .text_color(cx.theme().foreground)
                                                .whitespace_normal()
                                                .child(content),
                                        ),
                                )
                            })
                            .when(message_content.is_none() && file_count > 0, |this| {
                                this.child(
                                    div()
                                        .w_full()
                                        .max_w(px(760.))
                                        .mt(px(8.))
                                        .rounded_lg()
                                        .border_1()
                                        .border_color(cx.theme().border.opacity(0.7))
                                        .bg(cx.theme().secondary)
                                        .p(px(14.))
                                        .child(v_flex().gap(px(8.)).children(
                                            session.clone().into_iter().flat_map(|s| {
                                                s.items.into_iter().map(|item| {
                                                    let icon =
                                                        if item.file_type.starts_with("text/") {
                                                            "icons/book-open.svg"
                                                        } else {
                                                            "icons/file.svg"
                                                        };
                                                    h_flex()
                                                        .items_center()
                                                        .gap(px(8.))
                                                        .child(
                                                            Icon::default()
                                                                .path(icon)
                                                                .with_size(Size::Small)
                                                                .text_color(
                                                                    cx.theme().muted_foreground,
                                                                ),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_base()
                                                                .text_color(cx.theme().foreground)
                                                                .child(item.file_name),
                                                        )
                                                })
                                            }),
                                        )),
                                )
                            })
                            .when_some(message_content.clone(), |this, content| {
                                this.child(
                                    Button::new("receive-incoming-copy")
                                        .primary()
                                        .h(px(38.))
                                        .px(px(18.))
                                        .rounded_md()
                                        .mt(px(8.))
                                        .child("复制")
                                        .on_click(cx.listener(
                                            move |_this, _event, _window, cx| {
                                                if !content.is_empty() {
                                                    cx.write_to_clipboard(
                                                        ClipboardItem::new_string(content.clone()),
                                                    );
                                                }
                                            },
                                        )),
                                )
                            })
                            .when(show_cancelled, |this| {
                                this.child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().danger)
                                        .child("发送方已取消传输"),
                                )
                            }),
                    ),
            )
            .child(
                h_flex().w_full().justify_center().pb(px(26.)).child(
                    Button::new("receive-incoming-close")
                        .custom(close_button_variant)
                        .h(px(42.))
                        .px(px(16.))
                        .rounded_md()
                        .child(
                            h_flex()
                                .items_center()
                                .gap(px(8.))
                                .child(
                                    Icon::default()
                                        .path("icons/x.svg")
                                        .with_size(Size::Small)
                                        .text_color(hsla(0.0, 0.0, 1.0, 1.0)),
                                )
                                .child(
                                    div()
                                        .text_lg()
                                        .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                                        .child("关闭"),
                                ),
                        )
                        .on_click(cx.listener(|this, _e, window, cx| {
                            this.inbox_state.update(cx, |s, _| s.clear());
                            RouterState::global_mut(cx).location.pathname = "/".into();
                            window.refresh();
                        })),
                ),
            )
    }
}

fn visual_tag(fingerprint: &str) -> String {
    if fingerprint.is_empty() {
        return "--".to_string();
    }
    let mut sum: u32 = 0;
    for b in fingerprint.as_bytes() {
        sum = sum.wrapping_add(*b as u32);
    }
    format!("{:02}", (sum % 100))
}
