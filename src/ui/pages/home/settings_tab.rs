//! Settings tab: general, receive, send, network, other (uses ui/pages state types).

use super::HomePage;
use super::{ColorMode, ThemeMode};
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

/// Renders a boolean toggle entry (label + switch in a 150px muted container).
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
                .child(div().w(px(10.)))
                .child(
                    div()
                        .w(px(150.))
                        .h(px(50.))
                        .relative()
                        .child(div().absolute().inset_0().bg(cx.theme().muted).rounded_md())
                        .child(
                            div()
                                .absolute()
                                .inset_0()
                                .items_center()
                                .justify_center()
                                .child(
                                    Button::new(id.to_string())
                                        .ghost()
                                        .on_click(cx.listener(move |this, _ev, _win, _cx| {
                                            on_toggle(this);
                                        }))
                                        .child(Switch::new(value)),
                                ),
                        ),
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

/// Renders a dropdown-style entry (label + row of pre-built button elements).
fn render_dropdown_entry(
    label: &str,
    buttons: impl IntoElement,
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
                .child(
                    div().w(px(150.)).child(
                        div()
                            .bg(cx.theme().muted)
                            .rounded_md()
                            .p(px(8.))
                            .child(buttons),
                    ),
                ),
        )
        .into_any_element()
}

/// Helper: creates a toggle button for dropdown entries.
fn dropdown_btn(
    id: &str,
    label: &str,
    selected: bool,
) -> Button {
    let variant = if selected {
        gpui_component::button::ButtonVariant::Primary
    } else {
        gpui_component::button::ButtonVariant::Ghost
    };
    Button::new(id.to_string())
        .with_variant(variant)
        .child(label.to_string())
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
    let theme_mode = app.settings_state.theme_mode;
    let color_mode = app.settings_state.color_mode;
    let language = app.settings_state.language.clone();
    let server_running = app.settings_state.server_running;
    let server_alias = app.settings_state.server_alias.clone();
    let server_port = app.settings_state.server_port;
    let destination = app.settings_state.destination.clone();
    let network_filtered = app.settings_state.network_filtered;

    // -- General section --
    let brightness_buttons = h_flex()
        .justify_center()
        .gap(px(4.))
        .child(dropdown_btn("theme-system", "System", theme_mode == ThemeMode::System)
            .on_click(cx.listener(|this, _ev, _win, _cx| {
                this.settings_state.theme_mode = ThemeMode::System;
            })))
        .child(dropdown_btn("theme-light", "Light", theme_mode == ThemeMode::Light)
            .on_click(cx.listener(|this, _ev, _win, _cx| {
                this.settings_state.theme_mode = ThemeMode::Light;
            })))
        .child(dropdown_btn("theme-dark", "Dark", theme_mode == ThemeMode::Dark)
            .on_click(cx.listener(|this, _ev, _win, _cx| {
                this.settings_state.theme_mode = ThemeMode::Dark;
            })));

    let color_buttons = h_flex()
        .justify_center()
        .gap(px(4.))
        .child(dropdown_btn("color-system", "System", color_mode == ColorMode::System)
            .on_click(cx.listener(|this, _ev, _win, _cx| {
                this.settings_state.color_mode = ColorMode::System;
            })))
        .child(dropdown_btn("color-localsend", "NearSend", color_mode == ColorMode::LocalSend)
            .on_click(cx.listener(|this, _ev, _win, _cx| {
                this.settings_state.color_mode = ColorMode::LocalSend;
            })))
        .child(dropdown_btn("color-oled", "OLED", color_mode == ColorMode::Oled)
            .on_click(cx.listener(|this, _ev, _win, _cx| {
                this.settings_state.color_mode = ColorMode::Oled;
            })));

    // -- General section children (built separately to avoid multiple &mut cx borrows) --
    let g1 = render_dropdown_entry("Brightness", brightness_buttons, cx);
    let g2 = render_dropdown_entry("Color", color_buttons, cx);
    let g3 = render_button_entry("Language", &language, "language-btn", cx, |_this| {
        log::info!("Language clicked");
    });
    let animations = app.settings_state.animations;
    let g4 = render_boolean_entry("Animations", animations, "toggle-animations", cx, |this| {
        this.settings_state.animations = !this.settings_state.animations;
    });
    let general = render_settings_section("General", cx, vec![g1, g2, g3, g4]);

    // -- Receive section children --
    let quick_save = app.settings_state.quick_save;
    let quick_save_favorites = app.settings_state.quick_save_favorites;
    let require_pin = app.settings_state.require_pin;
    let save_to_gallery = app.settings_state.save_to_gallery;
    let auto_finish = app.settings_state.auto_finish;
    let save_to_history = app.settings_state.save_to_history;
    let dest_label = destination.clone().unwrap_or_else(|| "Downloads".to_string());

    let r1 = render_boolean_entry("Quick Save", quick_save, "toggle-quick-save", cx, |this| {
        this.settings_state.quick_save = !this.settings_state.quick_save;
    });
    let r2 = render_boolean_entry("Quick Save from Favorites", quick_save_favorites, "toggle-quick-save-favorites", cx, |this| {
        this.settings_state.quick_save_favorites = !this.settings_state.quick_save_favorites;
    });
    let r3 = render_boolean_entry("Require PIN", require_pin, "toggle-require-pin", cx, |this| {
        this.settings_state.require_pin = !this.settings_state.require_pin;
    });
    let r4 = render_button_entry("Destination", &dest_label, "destination", cx, |this| {
        this.settings_state.destination = Some("Downloads".to_string());
    });
    let r5 = render_boolean_entry("Save to Gallery", save_to_gallery, "toggle-save-to-gallery", cx, |this| {
        this.settings_state.save_to_gallery = !this.settings_state.save_to_gallery;
    });
    let r6 = render_boolean_entry("Auto Finish", auto_finish, "toggle-auto-finish", cx, |this| {
        this.settings_state.auto_finish = !this.settings_state.auto_finish;
    });
    let r7 = render_boolean_entry("Save to History", save_to_history, "toggle-save-to-history", cx, |this| {
        this.settings_state.save_to_history = !this.settings_state.save_to_history;
    });
    let receive = render_settings_section("Receive", cx, vec![r1, r2, r3, r4, r5, r6, r7]);

    // -- Send section (advanced only) --
    let send = if advanced {
        let share_via_link = app.settings_state.share_via_link_auto_accept;
        let s1 = render_boolean_entry("Share via Link Auto Accept", share_via_link, "toggle-share-via-link", cx, |this| {
            this.settings_state.share_via_link_auto_accept = !this.settings_state.share_via_link_auto_accept;
        });
        Some(render_settings_section("Send", cx, vec![s1]))
    } else {
        None
    };

    // -- Network section --
    let server_label = format!("Server{}", if server_running { "" } else { " (Offline)" });
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
                        .child(server_label.clone()),
                )
                .child(div().w(px(10.)))
                .child(
                    div().w(px(150.)).child(
                        div()
                            .bg(cx.theme().muted)
                            .rounded_md()
                            .child(
                                h_flex()
                                    .justify_center()
                                    .gap(px(4.))
                                    .child(
                                        Button::new("server-start")
                                            .ghost()
                                            .on_click(cx.listener(|this, _ev, _win, _cx| {
                                                this.settings_state.server_running = true;
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
                                            .on_click(cx.listener(|this, _ev, _win, _cx| {
                                                if this.settings_state.server_running {
                                                    this.settings_state.server_running = false;
                                                }
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

    let n1 = render_button_entry("Alias", &server_alias, "alias-input", cx, |_this| {
        log::info!("Alias input clicked");
    });
    let mut network_children: Vec<AnyElement> = vec![server_controls, n1];
    if advanced {
        let n2 = render_button_entry("Port", &server_port.to_string(), "port-input", cx, |_this| {
            log::info!("Port input clicked");
        });
        let encryption = app.settings_state.encryption;
        let n3 = render_boolean_entry("Encryption", encryption, "toggle-encryption", cx, |this| {
            this.settings_state.encryption = !this.settings_state.encryption;
        });
        let net_label = if network_filtered { "Filtered" } else { "All" };
        let n4 = render_button_entry("Network", net_label, "network", cx, |_this| {
            log::info!("Network clicked");
        });
        network_children.push(n2);
        network_children.push(n3);
        network_children.push(n4);
    }
    let network = render_settings_section(&server_label, cx, network_children);

    // -- Other section children --
    let o1 = render_button_entry("About", "Open", "about", cx, |_this| {
        log::info!("About clicked");
    });
    let o2 = render_button_entry("Support", "Donate", "donate", cx, |_this| {
        log::info!("Donate clicked");
    });
    let o3 = render_button_entry("Privacy Policy", "Open", "privacy", cx, |_this| {
        log::info!("Privacy clicked");
    });
    let other = render_settings_section("Other", cx, vec![o1, o2, o3]);

    // -- Advanced Settings toggle --
    let check_btn = Button::new("toggle-advanced-settings")
        .with_variant(if advanced {
            gpui_component::button::ButtonVariant::Primary
        } else {
            gpui_component::button::ButtonVariant::Secondary
        })
        .outline()
        .w(px(20.))
        .h(px(20.))
        .on_click(cx.listener(|this, _ev, _win, _cx| {
            this.settings_state.advanced = !this.settings_state.advanced;
        }));
    let check_btn = if advanced {
        check_btn.child(
            Icon::default()
                .path("icons/check.svg")
                .with_size(Size::XSmall)
                .text_color(cx.theme().primary_foreground),
        )
    } else {
        check_btn.child(div())
    };
    let advanced_toggle = h_flex()
        .justify_end()
        .w_full()
        .child(
            h_flex()
                .items_center()
                .gap(px(5.))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().foreground)
                        .child("Advanced Settings"),
                )
                .child(check_btn),
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
                .child("Changelog"),
        )
        .into_any_element();

    // -- Assemble page --
    let mut content = v_flex()
        .w_full()
        .px(px(15.))
        .py(px(40.))
        .gap(spacing::LG)
        .child(div().h(px(30.)))
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
