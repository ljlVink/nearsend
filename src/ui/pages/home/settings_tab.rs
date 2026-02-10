//! Settings tab: general, receive, send, network, other (uses ui/pages state types).

use super::HomePage;
use crate::ui::components::{logo::Logo, switch::Switch};
use crate::ui::theme::spacing;
use gpui::{div, prelude::*, px, AnyElement, Context, Window};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::select::Select;
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

/// Renders a select dropdown entry (label + Select component).
fn render_select_entry(
    label: &str,
    select: impl gpui::IntoElement,
    cx: &mut Context<HomePage>,
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
                .child(div().w(px(150.)).child(select)),
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
    let language = app.settings_state.language.clone();
    let server_running = app.settings_state.server_running;
    let server_alias = app.settings_state.server_alias.clone();
    let server_port = app.settings_state.server_port;
    let destination = app.settings_state.destination.clone();
    let network_filtered = app.settings_state.network_filtered;

    // -- General section children (built separately to avoid multiple &mut cx borrows) --
    let g1 = if let Some(ref state) = app.theme_select {
        render_select_entry("主题", Select::new(state).w(px(160.)), cx)
    } else {
        render_button_entry("主题", "系统", "theme-btn", cx, |_this| {})
    };
    let g2 = if let Some(ref state) = app.color_select {
        render_select_entry("颜色", Select::new(state).w(px(160.)), cx)
    } else {
        render_button_entry("颜色", "系统", "color-btn", cx, |_this| {})
    };
    let g3 = if let Some(ref state) = app.language_select {
        render_select_entry("语言", Select::new(state).w(px(160.)), cx)
    } else {
        render_button_entry("语言", &language, "language-btn", cx, |_this| {})
    };
    let animations = app.settings_state.animations;
    let g4 = render_boolean_entry("动画效果", animations, "toggle-animations", cx, |this| {
        this.settings_state.animations = !this.settings_state.animations;
    });
    let general = render_settings_section("常规", cx, vec![g1, g2, g3, g4]);

    // -- Receive section children --
    let quick_save = app.settings_state.quick_save;
    let quick_save_favorites = app.settings_state.quick_save_favorites;
    let require_pin = app.settings_state.require_pin;
    let save_to_gallery = app.settings_state.save_to_gallery;
    let auto_finish = app.settings_state.auto_finish;
    let save_to_history = app.settings_state.save_to_history;
    let dest_label = destination
        .clone()
        .unwrap_or_else(|| "Downloads".to_string());

    let r1 = render_boolean_entry("快速保存", quick_save, "toggle-quick-save", cx, |this| {
        this.settings_state.quick_save = !this.settings_state.quick_save;
    });
    let r2 = render_boolean_entry(
        "收藏夹快速保存",
        quick_save_favorites,
        "toggle-quick-save-favorites",
        cx,
        |this| {
            this.settings_state.quick_save_favorites = !this.settings_state.quick_save_favorites;
        },
    );
    let r3 = render_boolean_entry("需要 PIN", require_pin, "toggle-require-pin", cx, |this| {
        this.settings_state.require_pin = !this.settings_state.require_pin;
    });
    let r4 = render_button_entry("保存位置", &dest_label, "destination", cx, |this| {
        this.settings_state.destination = Some("Downloads".to_string());
    });
    let r5 = render_boolean_entry(
        "保存到相册",
        save_to_gallery,
        "toggle-save-to-gallery",
        cx,
        |this| {
            this.settings_state.save_to_gallery = !this.settings_state.save_to_gallery;
        },
    );
    let r6 = render_boolean_entry(
        "自动完成",
        auto_finish,
        "toggle-auto-finish",
        cx,
        |this| {
            this.settings_state.auto_finish = !this.settings_state.auto_finish;
        },
    );
    let r7 = render_boolean_entry(
        "保存到历史",
        save_to_history,
        "toggle-save-to-history",
        cx,
        |this| {
            this.settings_state.save_to_history = !this.settings_state.save_to_history;
        },
    );
    let receive = render_settings_section("接收", cx, vec![r1, r2, r3, r4, r5, r6, r7]);

    // -- Send section (advanced only) --
    let send = if advanced {
        let share_via_link = app.settings_state.share_via_link_auto_accept;
        let s1 = render_boolean_entry(
            "链接分享自动接受",
            share_via_link,
            "toggle-share-via-link",
            cx,
            |this| {
                this.settings_state.share_via_link_auto_accept =
                    !this.settings_state.share_via_link_auto_accept;
            },
        );
        Some(render_settings_section("发送", cx, vec![s1]))
    } else {
        None
    };

    // -- Network section --
    let server_label_text = format!("服务器{}", if server_running { "" } else { " (离线)" });
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
                                    Button::new("server-start")
                                        .ghost()
                                        .on_click(cx.listener(|this, _ev, _win, cx| {
                                            this.start_local_server(cx);
                                        }))
                                        .child(
                                            Icon::default()
                                                .path("icons/refresh.svg")
                                                .with_size(Size::Small),
                                        ),
                                )
                                .child(
                                    Button::new("server-stop")
                                        .ghost()
                                        .on_click(cx.listener(|this, _ev, _win, cx| {
                                            this.stop_local_server(cx);
                                        }))
                                        .child(
                                            Icon::default()
                                                .path("icons/stop.svg")
                                                .with_size(Size::Small),
                                        ),
                                ),
                        ),
                    ),
                ),
        )
        .into_any_element();

    let n1 = render_button_entry("别名", &server_alias, "alias-input", cx, |_this| {
        log::info!("Alias input clicked");
    });
    let mut network_children: Vec<AnyElement> = vec![server_controls, n1];
    if advanced {
        let n2 = render_button_entry(
            "端口",
            &server_port.to_string(),
            "port-input",
            cx,
            |_this| {
                log::info!("Port input clicked");
            },
        );
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
        network_children.push(n2);
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
                        .child(Switch::new(advanced)),
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
        .gap(spacing::LG)
        .child(general)
        .child(receive);

    if let Some(send) = send {
        content = content.child(send);
    }

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
