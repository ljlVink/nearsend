use crate::state::{
    app_state::AppState, receive_inbox_state::ReceiveInboxState, transfer_state::TransferDirection,
};
use gpui::{div, hsla, prelude::*, px, Context, Entity, Window};
use gpui_component::{
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    h_flex, v_flex, ActiveTheme as _, Icon, Sizable as _, Size,
};
use gpui_router::RouterState;
use std::collections::HashSet;

pub struct ReceiveIncomingPage {
    app_state: Entity<AppState>,
    inbox_state: Entity<ReceiveInboxState>,
}

impl ReceiveIncomingPage {
    pub fn new(app_state: Entity<AppState>, inbox_state: Entity<ReceiveInboxState>) -> Self {
        Self {
            app_state,
            inbox_state,
        }
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
            if s.is_message_only {
                s.items.first().and_then(|item| item.text_content.clone())
            } else {
                None
            }
        });
        let file_count = session.as_ref().map(|s| s.items.len()).unwrap_or(0);
        let direction = session
            .as_ref()
            .map(|s| s.direction)
            .unwrap_or(TransferDirection::Receive);
        let subtitle = if message_content.is_some() {
            if direction == TransferDirection::Send {
                format!("你发送给 {} 的消息：", sender_alias)
            } else {
                "发送给你了一条消息：".to_string()
            }
        } else if file_count > 0 {
            if direction == TransferDirection::Send {
                format!("你发送给 {} {} 个文件", sender_alias, file_count)
            } else {
                format!("发送给你 {} 个文件", file_count)
            }
        } else {
            "等待接收内容".to_string()
        };
        let show_cancelled = session.as_ref().map(|s| s.cancelled).unwrap_or(false);
        let show_waiting_actions = session
            .as_ref()
            .map(|s| !s.completed && !s.cancelled && !s.is_message_only)
            .unwrap_or(false);
        let selected_file_ids: HashSet<String> = session
            .as_ref()
            .map(|s| s.selected_file_ids.iter().cloned().collect())
            .unwrap_or_default();
        let active_session_id = session.as_ref().map(|s| s.session_id.clone());
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
                                                    let file_id = item.file_id.clone();
                                                    let icon =
                                                        if item.file_type.starts_with("text/") {
                                                            "icons/book-open.svg"
                                                        } else {
                                                            "icons/file.svg"
                                                        };
                                                    let selected =
                                                        selected_file_ids.contains(&item.file_id);
                                                    Button::new(format!(
                                                        "receive-file-item-{}",
                                                        item.file_id
                                                    ))
                                                        .custom(
                                                            ButtonCustomVariant::new(cx)
                                                                .color(if selected {
                                                                    cx.theme().primary.opacity(0.10)
                                                                } else {
                                                                    cx.theme().secondary
                                                                })
                                                                .foreground(cx.theme().foreground)
                                                                .hover(if selected {
                                                                    cx.theme().primary.opacity(0.15)
                                                                } else {
                                                                    cx.theme().secondary
                                                                })
                                                                .active(if selected {
                                                                    cx.theme().primary.opacity(0.20)
                                                                } else {
                                                                    cx.theme().secondary
                                                                }),
                                                        )
                                                        .w_full()
                                                        .h(px(36.))
                                                        .justify_start()
                                                        .when(show_waiting_actions, |this| {
                                                            this.on_click(cx.listener(
                                                                move |this, _e, _window, cx| {
                                                                    this.inbox_state.update(
                                                                        cx,
                                                                        |state, _| {
                                                                            state.toggle_file_selected(&file_id);
                                                                        },
                                                                    );
                                                                },
                                                            ))
                                                        })
                                                        .child(
                                                            Icon::default()
                                                                .path(
                                                                    if selected {
                                                                        "icons/check.svg"
                                                                    } else {
                                                                        "icons/x.svg"
                                                                    },
                                                                )
                                                                .with_size(Size::XSmall)
                                                                .text_color(if selected {
                                                                    cx.theme().primary
                                                                } else {
                                                                    cx.theme().muted_foreground
                                                                }),
                                                        )
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
                                            move |this, _event, _window, cx| {
                                                if !content.is_empty() {
                                                    let tokio_handle =
                                                        this.app_state.read(cx).tokio_handle.clone();
                                                    let content_to_write = content.clone();
                                                    tokio_handle.spawn(async move {
                                                        if let Err(err) =
                                                            crate::platform::clipboard::write_clipboard_text(
                                                                content_to_write,
                                                            )
                                                            .await
                                                        {
                                                            log::error!(
                                                                "write clipboard text failed: {}",
                                                                err
                                                            );
                                                        }
                                                    });
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
            .when(show_waiting_actions, |this| {
                let session_id = active_session_id.clone().unwrap_or_default();
                let session_id_for_decline = session_id.clone();
                let session_id_for_accept = session_id.clone();
                let selected_ids_for_accept = session
                    .as_ref()
                    .map(|s| s.selected_file_ids.clone())
                    .unwrap_or_default();
                this.child(
                    h_flex()
                        .w_full()
                        .justify_center()
                        .gap(px(12.))
                        .pb(px(10.))
                        .child(
                            Button::new("receive-incoming-decline")
                                .outline()
                                .h(px(40.))
                                .px(px(18.))
                                .rounded_md()
                                .child("拒绝")
                                .on_click(cx.listener(move |this, _e, window, cx| {
                                    crate::core::receive_events::submit_incoming_decision(
                                        session_id_for_decline.clone(),
                                        crate::core::receive_events::IncomingTransferDecision::Decline,
                                    );
                                    this.inbox_state.update(cx, |s, _| s.clear());
                                    RouterState::global_mut(cx).location.pathname = "/".into();
                                    window.refresh();
                                })),
                        )
                        .child(
                            Button::new("receive-incoming-accept")
                                .primary()
                                .h(px(40.))
                                .px(px(18.))
                                .rounded_md()
                                .child("接受")
                                .on_click(cx.listener(move |_this, _e, _window, _cx| {
                                    crate::core::receive_events::submit_incoming_decision(
                                        session_id_for_accept.clone(),
                                        crate::core::receive_events::IncomingTransferDecision::AcceptSelected(
                                            selected_ids_for_accept.clone(),
                                        ),
                                    );
                                })),
                        ),
                )
            })
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
                        .on_click(cx.listener(move |this, _e, window, cx| {
                            if let Some(active) = this.inbox_state.read(cx).active.as_ref() {
                                if !active.completed && !active.cancelled && !active.is_message_only {
                                    crate::core::receive_events::submit_incoming_decision(
                                        active.session_id.clone(),
                                        crate::core::receive_events::IncomingTransferDecision::Decline,
                                    );
                                }
                            }
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
