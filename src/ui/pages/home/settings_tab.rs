//! Settings tab: general, receive, send, network, other (uses ui/pages state types).

use super::HomePage;
use crate::ui::components::{logo::Logo, switch::Switch};
use crate::ui::theme::spacing;
use gpui::{div, prelude::*, px, AnyElement, Context, Window};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex, v_flex, ActiveTheme as _, Icon, Sizable as _, Size, StyledExt as _,
};

// ---------------------------------------------------------------------------
// Reusable helpers
// ---------------------------------------------------------------------------

/// Renders a settings section card with a title and a list of child entries.
fn render_settings_section(
    title: &str,
    cx: &mut Context<HomePage>,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut inner = v_flex().gap(px(10.)).child(
        div()
            .text_lg()
            .font_semibold()
            .text_color(cx.theme().foreground)
            .child(title.to_string()),
    );
    for child in children {
        inner = inner.child(child);
    }
    div()
        .bg(cx.theme().secondary)
        .border_1()
        .border_color(cx.theme().border)
        .rounded_lg()
        .p(px(15.))
        .child(inner)
        .into_any_element()
}

/// Renders a boolean toggle entry (label + switch).
fn render_boolean_entry(
    label: &str,
    value: bool,
    id: &str,
    cx: &mut Context<HomePage>,
    on_toggle: impl Fn(&mut HomePage) + 'static,
) -> AnyElement {
    div()
        .pb(px(15.))
        .child(
            h_flex()
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().foreground)
                        .flex_1()
                        .child(label.to_string()),
                )
                .child(
                    div()
                        .id(id.to_string())
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _ev, _win, _cx| {
                            on_toggle(this);
                        }))
                        .child(Switch::new(value)),
                ),
        )
        .into_any_element()
}

/// Renders a button entry (label + secondary outline button in a 150px container).
fn render_button_entry(
    label: &str,
    button_text: &str,
    id: &str,
    cx: &mut Context<HomePage>,
    on_click: impl Fn(&mut HomePage) + 'static,
) -> AnyElement {
    div()
        .pb(px(15.))
        .child(
            h_flex()
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().foreground)
                        .flex_1()
                        .child(label.to_string()),
                )
                .child(div().w(px(10.)))
                .child(
                    div().w(px(150.)).child(
                        Button::new(id.to_string())
                            .with_variant(gpui_component::button::ButtonVariant::Secondary)
                            .outline()
                            .w_full()
                            .on_click(cx.listener(move |this, _ev, _win, _cx| {
                                on_click(this);
                            }))
                            .child(button_text.to_string()),
                    ),
                ),
        )
        .into_any_element()
}

/// Renders a button entry whose click handler needs `window` and `cx`.
fn render_clickable_entry(
    label: &str,
    button_text: &str,
    id: &str,
    cx: &mut Context<HomePage>,
    on_click: impl Fn(&mut HomePage, &mut Window, &mut Context<HomePage>) + 'static,
) -> AnyElement {
    div()
        .pb(px(15.))
        .child(
            h_flex()
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().foreground)
                        .flex_1()
                        .child(label.to_string()),
                )
                .child(div().w(px(10.)))
                .child(
                    div().w(px(150.)).child(
                        Button::new(id.to_string())
                            .with_variant(gpui_component::button::ButtonVariant::Secondary)
                            .outline()
                            .w_full()
                            .on_click(cx.listener(move |this, _ev, window, cx| {
                                on_click(this, window, cx);
                            }))
                            .child(button_text.to_string()),
                    ),
                ),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Main render
// ---------------------------------------------------------------------------

pub fn render_settings_content(
    app: &mut HomePage,
    _window: &mut Window,
    cx: &mut Context<HomePage>,
) -> AnyElement {
    let advanced = app.settings_state.advanced;
    let server_running = app.settings_state.server_running;
    let server_paused = app.settings_state.server_paused;
    let server_alias = app.settings_state.server_alias.clone();
    let server_port = app.settings_state.server_port;
    let network_filtered = app.settings_state.network_filtered;

    // "常规" 和 "接收" 能力暂未支持，先在设置页隐藏。

    // "发送" 能力暂未支持，先在设置页隐藏。

    // -- Network section --
    let server_label_text = format!("服务器{}", if server_running { "" } else { " (离线)" });
    let can_pause = server_running && !server_paused;
    let server_controls = div()
        .pb(px(15.))
        .child(
            h_flex()
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().foreground)
                        .flex_1()
                        .child(server_label_text.clone()),
                )
                .child(div().w(px(10.)))
                .child(
                    div().w(px(150.)).child(
                        div().bg(cx.theme().muted).rounded_md().child(
                            h_flex()
                                .justify_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .id("server-start")
                                        .cursor_pointer()
                                        .px(px(8.))
                                        .py(px(6.))
                                        .rounded_md()
                                        .on_click(cx.listener(|this, _ev, _win, cx| {
                                            if this.settings_state.server_paused {
                                                this.resume_local_server(cx);
                                            } else if this.settings_state.server_running {
                                                this.restart_local_server_with_current_config(cx);
                                            } else {
                                                this.start_local_server(cx);
                                            }
                                        }))
                                        .when(server_paused, |this| {
                                            this.child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .text_color(cx.theme().foreground)
                                                    .child("▶"),
                                            )
                                        })
                                        .when(!server_paused, |this| {
                                            this.child(
                                                Icon::default()
                                                    .path("icons/refresh.svg")
                                                    .with_size(Size::Small)
                                                    .text_color(cx.theme().foreground),
                                            )
                                        }),
                                )
                                .child(
                                    div()
                                        .id("server-stop")
                                        .px(px(8.))
                                        .py(px(6.))
                                        .rounded_md()
                                        .when(can_pause, |this| {
                                            this.cursor_pointer().on_click(cx.listener(
                                                |this, _ev, _win, cx| {
                                                    this.pause_local_server(cx);
                                                },
                                            ))
                                        })
                                        .child(div().w(px(12.)).h(px(12.)).rounded_sm().bg(
                                            if can_pause {
                                                cx.theme().foreground
                                            } else {
                                                cx.theme().muted_foreground.opacity(0.35)
                                            },
                                        )),
                                ),
                        ),
                    ),
                ),
        )
        .into_any_element();

    let n1 = render_clickable_entry(
        "别名",
        &server_alias,
        "alias-input",
        cx,
        |this, window, cx| {
            this.open_server_alias_dialog(window, cx);
        },
    );
    let n2 = render_clickable_entry(
        "端口",
        &server_port.to_string(),
        "port-input",
        cx,
        |this, window, cx| {
            this.open_server_port_dialog(window, cx);
        },
    );
    let mut network_children: Vec<AnyElement> = vec![server_controls, n1, n2];
    if advanced {
        let encryption = app.settings_state.encryption;
        let n3 = render_boolean_entry("加密", encryption, "toggle-encryption", cx, |this| {
            this.settings_state.encryption = !this.settings_state.encryption;
        });
        let net_label = if network_filtered {
            "已过滤"
        } else {
            "全部"
        };
        let n4 = render_button_entry("网络", net_label, "network", cx, |_this| {
            log::info!("Network clicked");
        });
        network_children.push(n3);
        network_children.push(n4);
    }
    let network = render_settings_section("网络", cx, network_children);

    // -- Other section children --
    let o1 = render_button_entry("关于", "打开", "about", cx, |_this| {
        log::info!("About clicked");
    });
    let o2 = render_button_entry("支持", "捐赠", "donate", cx, |_this| {
        log::info!("Donate clicked");
    });
    let o3 = render_button_entry("隐私政策", "打开", "privacy", cx, |_this| {
        log::info!("Privacy clicked");
    });
    let other = render_settings_section("其他", cx, vec![o1, o2, o3]);

    // -- Advanced Settings toggle --
    let advanced_toggle = h_flex()
        .justify_end()
        .w_full()
        .child(
            h_flex()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().foreground)
                        .child("高级设置"),
                )
                .child(
                    div()
                        .id("toggle-advanced-settings")
                        .cursor_pointer()
                        .on_click(cx.listener(|this, _ev, _win, _cx| {
                            this.settings_state.advanced = !this.settings_state.advanced;
                        }))
                        .child(
                            div()
                                .w(px(18.))
                                .h(px(18.))
                                .rounded(px(4.))
                                .border_1()
                                .border_color(if advanced {
                                    cx.theme().primary
                                } else {
                                    cx.theme().border
                                })
                                .bg(if advanced {
                                    cx.theme().primary
                                } else {
                                    cx.theme().background
                                })
                                .flex()
                                .items_center()
                                .justify_center()
                                .when(advanced, |this| {
                                    this.child(
                                        Icon::default()
                                            .path("icons/check.svg")
                                            .with_size(Size::XSmall)
                                            .text_color(cx.theme().primary_foreground),
                                    )
                                }),
                        ),
                ),
        )
        .into_any_element();

    // -- About section --
    let about = v_flex()
        .gap(px(5.))
        .items_center()
        .child(Logo::new().size(80.).with_text(true))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .text_center()
                .child("Version 0.1.0"),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .text_center()
                .child("\u{00a9} 2025 NearSend"),
        )
        .child(
            Button::new("changelog")
                .ghost()
                .on_click(cx.listener(|_this, _ev, _win, _cx| {
                    log::info!("Changelog clicked");
                }))
                .child("更新日志"),
        )
        .into_any_element();

    // -- Assemble page --
    let mut content = v_flex()
        .w_full()
        .px(px(15.))
        .pt(px(15.))
        .pb(px(40.))
        .gap(spacing::LG);

    content = content
        .child(network)
        .child(other)
        .child(advanced_toggle)
        .child(about)
        .child(div().h(px(80.)));

    div()
        .size_full()
        .w_full()
        .bg(cx.theme().background)
        .overflow_y_scrollbar()
        .child(content)
        .into_any_element()
}
