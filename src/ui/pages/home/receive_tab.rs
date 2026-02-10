//! Receive tab content for home page (top bar, middle logo, bottom Quick Save bar).

use super::QuickSaveMode;
use super::HomePage;
use crate::ui::components::logo::Logo;
use crate::ui::components::rotating_widget::RotatingWidget;
use crate::ui::theme::spacing;
use gpui::{div, prelude::*, px, AnyElement, Context, Window};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::{
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    h_flex,
    popover::Popover,
    tab::{Tab, TabBar},
    v_flex, ActiveTheme as _, Anchor, Icon, Sizable as _, Size, StyledExt as _,
};
use gpui_router::RouterState;

pub fn render_receive_content(
    home: &mut HomePage,
    _window: &mut Window,
    cx: &mut Context<HomePage>,
) -> AnyElement {
    let show_advanced = home.receive_state.show_advanced;
    let quick_save_mode = home.receive_state.quick_save_mode;
    let server_alias = home.receive_state.server_alias.clone();
    let server_ips = home.receive_state.server_ips.clone();
    let server_port = home.receive_state.server_port;
    let server_running = home.receive_state.server_running;
    let animations = home.settings_state.animations;
    let home_entity = cx.entity();

    let info_alias = server_alias.clone();
    let info_ips = server_ips.clone();

    v_flex()
        .size_full()
        .bg(cx.theme().background)
        // Top bar: history + info buttons
        .child(
            h_flex()
                .w_full()
                .h(px(56.))
                .px(px(20.))
                .items_center()
                .justify_end()
                .gap(spacing::SM)
                // History button — hidden when info popover is open
                .when(!show_advanced, |this| {
                    this.child(render_circle_button(
                        "receive-history",
                        "icons/history.svg",
                        cx,
                        |_this, _event, window, cx| {
                            RouterState::global_mut(cx).location.pathname =
                                "/receive/history".into();
                            window.refresh();
                        },
                    ))
                })
                // Info popover
                .child(
                    Popover::new("receive-info")
                        .anchor(Anchor::TopRight)
                        .overlay_closable(false)
                        .open(show_advanced)
                        .on_open_change(move |open, _window, cx| {
                            home_entity.update(cx, |this, _cx| {
                                this.receive_state.show_advanced = *open;
                            });
                        })
                        .trigger(
                            Button::new("receive-info")
                                .custom(
                                    ButtonCustomVariant::new(cx)
                                        .hover(cx.theme().transparent)
                                        .active(cx.theme().transparent),
                                )
                                .rounded_full()
                                .p(px(8.))
                                .child(
                                    div()
                                        .shadow_xs()
                                        .rounded_full()
                                        .w(px(44.))
                                        .h(px(44.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            Icon::default()
                                                .path("icons/info.svg")
                                                .with_size(Size::Medium),
                                        ),
                                ),
                        )
                        .content(move |_state, _window, cx| {
                            v_flex()
                                .gap(spacing::SM)
                                .child(render_info_row("Alias:", &info_alias, cx))
                                .child(
                                    h_flex()
                                        .items_start()
                                        .child(
                                            div()
                                                .w(px(60.))
                                                .text_sm()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("IP:"),
                                        )
                                        .child(if info_ips.is_empty() {
                                            div()
                                                .text_sm()
                                                .text_color(cx.theme().foreground)
                                                .child("Unknown")
                                        } else {
                                            v_flex().gap(px(2.)).items_start().children(
                                                info_ips.iter().map(|ip| {
                                                    div()
                                                        .text_sm()
                                                        .text_color(cx.theme().foreground)
                                                        .child(ip.clone())
                                                }),
                                            )
                                        }),
                                )
                                .child(render_info_row(
                                    "Port:",
                                    &server_port.to_string(),
                                    cx,
                                ))
                        }),
                ),
        )
        // Middle: logo + alias + IP display
        .child(
            div()
                .flex_1()
                .min_h(px(0.))
                .w_full()
                .overflow_y_scrollbar()
                .child(
                    div().w_full().max_w(px(600.)).p(px(30.)).h_full().child(
                        v_flex()
                            .flex_1()
                            .items_center()
                            .justify_center()
                            .gap(spacing::MD)
                            .child(
                                RotatingWidget::new(
                                    div()
                                        .w(px(200.))
                                        .h(px(200.))
                                        .flex_none()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(Logo::default().size(200.)),
                                )
                                .spinning(server_running && animations)
                                .duration(15),
                            )
                            .child(
                                div()
                                    .max_w(px(520.))
                                    .text_3xl()
                                    .font_bold()
                                    .text_color(cx.theme().foreground)
                                    .text_center()
                                    .child(server_alias.clone()),
                            )
                            .child(
                                div()
                                    .max_w(px(520.))
                                    .text_xl()
                                    .text_color(cx.theme().muted_foreground)
                                    .text_center()
                                    .child(if server_running && !server_ips.is_empty() {
                                        server_ips
                                            .iter()
                                            .map(|ip| format!("#{}", ip))
                                            .collect::<Vec<_>>()
                                            .join(" ")
                                    } else {
                                        "Offline".to_string()
                                    }),
                            ),
                    ),
                ),
        )
        // Bottom: Quick Save selector
        .child(
            div()
                .w_full()
                .pt(px(20.))
                .pb(px(15.))
                .px(px(30.))
                .child(
                    v_flex()
                        .gap(spacing::MD)
                        .items_center()
                        .child(
                            div()
                                .text_lg()
                                .font_medium()
                                .text_color(cx.theme().foreground)
                                .child("自动保存"),
                        )
                        .child(
                            TabBar::new("quick-save")
                                .w_full()
                                .segmented()
                                .with_size(Size::Large)
                                .selected_index(match quick_save_mode {
                                    QuickSaveMode::Off => 0,
                                    QuickSaveMode::Favorites => 1,
                                    QuickSaveMode::On => 2,
                                })
                                .on_click(cx.listener(|this, index, _window, _cx| {
                                    this.receive_state.quick_save_mode = match *index {
                                        0 => QuickSaveMode::Off,
                                        1 => QuickSaveMode::Favorites,
                                        2 => QuickSaveMode::On,
                                        _ => QuickSaveMode::Off,
                                    };
                                }))
                                .children([
                                    Tab::new().flex_1().label("关"),
                                    Tab::new().flex_1().label("收藏夹"),
                                    Tab::new().flex_1().label("开"),
                                ]),
                        ),
                ),
        )
        .into_any_element()
}

/// Render a single info row with fixed-width label.
fn render_info_row(label: &str, value: &str, cx: &gpui::App) -> impl IntoElement {
    h_flex()
        .items_start()
        .child(
            div()
                .w(px(60.))
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(label.to_string()),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().foreground)
                .child(value.to_string()),
        )
}

/// Render a circular icon button (used for history, info, etc.).
fn render_circle_button(
    id: &str,
    icon_path: &str,
    cx: &Context<HomePage>,
    on_click: impl Fn(&mut HomePage, &gpui::ClickEvent, &mut Window, &mut Context<HomePage>) + 'static,
) -> impl IntoElement {
    let icon_path = icon_path.to_string();
    div()
        .id(id.to_string())
        .cursor_default()
        .rounded_full()
        .p(px(8.))
        .child(
            div()
                .shadow_xs()
                .rounded_full()
                .w(px(44.))
                .h(px(44.))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Icon::default()
                        .path(icon_path)
                        .with_size(Size::Medium),
                ),
        )
        .on_click(cx.listener(on_click))
}