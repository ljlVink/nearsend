//! Send tab: select content type, files, and nearby devices (LocalSend-aligned layout).

use super::HomePage;
use crate::ui::components::{
    device_card::DeviceCard, device_placeholder::DevicePlaceholder,
    opacity_slideshow::OpacitySlideshow, rotating_widget::RotatingWidget,
};
use crate::ui::theme::spacing;
use crate::ui::utils::format_file_size;
use gpui::{div, prelude::*, px, AnyElement, Context, ScrollHandle, Window};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::{
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    h_flex, v_flex, ActiveTheme as _, Icon, Sizable as _, Size, StyledExt as _,
};

/// Button width/height for content type buttons (matches BigButton constants).
const CONTENT_BTN_WIDTH: f32 = 90.0;
const CONTENT_BTN_HEIGHT: f32 = 65.0;
const CONTENT_BTN_GAP: f32 = 10.0;
const CONTENT_BTN_COUNT: f32 = 4.0;
const CONTENT_ROW_MIN_WIDTH: f32 =
    CONTENT_BTN_WIDTH * CONTENT_BTN_COUNT + CONTENT_BTN_GAP * (CONTENT_BTN_COUNT - 1.0) + 2.0;

/// Render a content-type selector button (File / Media / Text / Folder).
/// Primary background + white text, no hover/active state change.
fn render_content_type_button(
    id: impl Into<gpui::ElementId>,
    icon_path: impl Into<gpui::SharedString>,
    label: &str,
    selected: bool,
    cx: &mut Context<HomePage>,
    on_click: impl Fn(&mut HomePage, &mut Window, &mut Context<HomePage>) + 'static,
) -> AnyElement {
    let icon_path = icon_path.into();
    let primary = cx.theme().primary;
    let bg = if selected { primary } else { cx.theme().secondary };
    let fg = if selected {
        cx.theme().primary_foreground
    } else {
        cx.theme().foreground
    };

    Button::new(id)
        .flex_none()
        .custom(
            ButtonCustomVariant::new(cx)
                .color(bg)
                .foreground(fg)
                .hover(bg)
                .active(bg),
        )
        .w(px(CONTENT_BTN_WIDTH))
        .h(px(CONTENT_BTN_HEIGHT))
        .rounded_md()
        .on_click(cx.listener(move |this, _event, window, cx| {
            on_click(this, window, cx);
        }))
        .child(
            v_flex()
                .items_center()
                .justify_between()
                .gap(px(4.))
                .child(
                    Icon::default()
                        .path(icon_path.clone())
                        .with_size(gpui_component::Size::Medium)
                        .text_color(fg),
                )
                .child(
                    div()
                        .text_sm()
                        .text_center()
                        .text_color(fg)
                        .child(label.to_string()),
                ),
        )
        .into_any_element()
}

/// Render a circular action button (scan / send / favorites / settings).
fn render_action_button(
    id: impl Into<gpui::ElementId>,
    icon_path: impl Into<gpui::SharedString>,
    cx: &mut Context<HomePage>,
    on_click: impl Fn(&mut HomePage, &mut Window, &mut Context<HomePage>) + 'static,
) -> AnyElement {
    let icon_path = icon_path.into();
    div()
        .id(id)
        .cursor_default()
        .rounded_full()
        .p(px(4.))
        .child(
            div()
                .shadow_xs()
                .rounded_full()
                .w(px(38.))
                .h(px(38.))
                .flex()
                .items_center()
                .justify_center()
                .child(Icon::default().path(icon_path).with_size(Size::Small)),
        )
        .on_click(cx.listener(move |this, _event, window, cx| {
            on_click(this, window, cx);
        }))
        .into_any_element()
}

pub fn render_send_content(
    app: &mut HomePage,
    window: &mut Window,
    cx: &mut Context<HomePage>,
) -> AnyElement {
    if !app.send_state.has_scanned_once
        && !app.send_state.scanning
        && app.send_state.nearby_devices.is_empty()
    {
        app.start_discovery_scan(cx);
    }

    let selected_files = app.send_state.selected_files.clone();
    let has_files = !selected_files.is_empty();
    let scanning = app.send_state.scanning;
    let send_content_type = app.send_state.send_content_type;
    let total_size = app.send_state.selected_files_total_size;
    let animations = app.settings_state.animations;
    let home_entity = cx.entity();
    let select_row_scroll = window
        .use_keyed_state("send-select-row-scroll", cx, |_, _| {
            ScrollHandle::default()
        })
        .read(cx)
        .clone();

    v_flex()
    .size_full()
    .bg(cx.theme().background)
    .relative()
    .child(
        div()
            .size_full()
            .w_full()
            .overflow_y_scrollbar()
            .child(
                v_flex()
                    .w_full()
                    .pt(px(20.))
                    .child(
                v_flex()
                    .w_full()
                    .gap(spacing::SM)
                    // -- Section title: "选择" --
                    .child(
                        div()
                            .px(px(15.))
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::BLACK)
                                    .text_color(cx.theme().foreground)
                                    .child("选择"),
                            ),
                    )
                    // -- Content type buttons row --
                    .child(
                        div()
                            .px(px(15.))
                            .child(
                                div()
                                    .id("send-content-type-scroll")
                                    .w_full()
                                    .track_scroll(&select_row_scroll)
                                    .overflow_x_scroll()
                                    .child(
                                        h_flex()
                                            .min_w(px(CONTENT_ROW_MIN_WIDTH))
                                            .gap(px(CONTENT_BTN_GAP))
                                            .items_center()
                                            .child(render_content_type_button(
                                                "content-file",
                                                "icons/file.svg",
                                                "文件",
                                                send_content_type == super::SendContentType::File,
                                                cx,
                                                |this, window, cx| {
                                                    this.handle_pick_content(
                                                        super::SendContentType::File,
                                                        window,
                                                        cx,
                                                    );
                                                },
                                            ))
                                            .child(render_content_type_button(
                                                "content-folder",
                                                "icons/folder.svg",
                                                "文件夹",
                                                send_content_type == super::SendContentType::Folder,
                                                cx,
                                                |this, window, cx| {
                                                    this.handle_pick_content(
                                                        super::SendContentType::Folder,
                                                        window,
                                                        cx,
                                                    );
                                                },
                                            ))
                                            .child(render_content_type_button(
                                                "content-text",
                                                "icons/book-open.svg",
                                                "文本",
                                                send_content_type == super::SendContentType::Text,
                                                cx,
                                                |this, window, cx| {
                                                    this.handle_pick_content(
                                                        super::SendContentType::Text,
                                                        window,
                                                        cx,
                                                    );
                                                },
                                            ))
                                            .child(render_content_type_button(
                                                "content-clipboard",
                                                "icons/external-link.svg",
                                                "剪贴板",
                                                send_content_type == super::SendContentType::Media,
                                                cx,
                                                |this, window, cx| {
                                                    this.handle_pick_content(
                                                        super::SendContentType::Media,
                                                        window,
                                                        cx,
                                                    );
                                                },
                                            )),
                                    ),
                            ),
                    )
                    // -- Selected files card --
                    .when(has_files, |this| {
                        let file_count = selected_files.len();
                        this.child(
                            div()
                                .mx(px(15.))
                                .mb(px(10.))
                                .bg(cx.theme().secondary)
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_lg()
                                .pl(px(15.))
                                .pt(px(5.))
                                .pb(px(15.))
                                .child(
                                    v_flex()
                                        .gap(spacing::MD)
                                        // Card title: file count
                                        .child(
                                            h_flex()
                                                .justify_between()
                                                .items_center()
                                                .child(
                                                    div()
                                                        .text_base()
                                                        .font_weight(gpui::FontWeight::BLACK)
                                                        .text_color(cx.theme().foreground)
                                                        .child(format!("{} 个文件已选择", file_count)),
                                                )
                                                .child(
                                                    Button::new("clear")
                                                        .ghost()
                                                        .on_click(cx.listener(|this, _event, _window, _cx| {
                                                            this.send_state.selected_files.clear();
                                                        }))
                                                        .child(
                                                            Icon::default()
                                                                .path("icons/close.svg")
                                                                .with_size(gpui_component::Size::Small),
                                                        ),
                                                ),
                                        )
                                        // File count + total size
                                        .child(
                                            v_flex()
                                                .gap(spacing::XS)
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(cx.theme().foreground)
                                                        .child(format!("{} 个文件", file_count)),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(format_file_size(total_size)),
                                                ),
                                        )
                                        // File thumbnails
                                        .child(
                                            div()
                                                .h(px(80.))
                                                .overflow_x_scrollbar()
                                                .child(
                                                    h_flex()
                                                        .gap(px(10.))
                                                        .children(selected_files.iter().map(|file| {
                                                            div()
                                                                .pr(px(10.))
                                                                .child(
                                                                    v_flex()
                                                                        .w(px(80.))
                                                                        .h(px(80.))
                                                                        .bg(cx.theme().muted)
                                                                        .rounded_md()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .gap(px(4.))
                                                                        .child(
                                                                            Icon::default()
                                                                                .path("icons/file.svg")
                                                                                .with_size(gpui_component::Size::Medium),
                                                                        )
                                                                        .child(
                                                                            div()
                                                                                .text_xs()
                                                                                .text_color(cx.theme().muted_foreground)
                                                                                .max_w(px(72.))
                                                                                .overflow_hidden()
                                                                                .child(file.name.clone()),
                                                                        ),
                                                                )
                                                        })),
                                                ),
                                        )
                                        // Edit / Add buttons
                                        .child(
                                            h_flex()
                                                .justify_end()
                                                .gap(px(15.))
                                                .child(
                                                    Button::new("edit")
                                                        .ghost()
                                                        .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                            log::info!("Edit clicked");
                                                        }))
                                                        .child("编辑"),
                                                )
                                                .child(
                                                    Button::new("add")
                                                        .with_variant(gpui_component::button::ButtonVariant::Primary)
                                                        .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                            log::info!("Add clicked");
                                                        }))
                                                        .child(
                                                            h_flex()
                                                                .items_center()
                                                                .gap(px(6.))
                                                                .child(
                                                                    Icon::default()
                                                                        .path("icons/plus.svg")
                                                                        .with_size(gpui_component::Size::Small),
                                                                )
                                                                .child("添加"),
                                                        ),
                                                ),
                                        ),
                                ),
                        )
                    })
                    // -- Nearby devices section --
                    .child(
                        div()
                            .px(px(15.))
                            .pt(px(5.))
                            .child(
                                h_flex()
                                    .gap(px(16.))
                                    .items_center()
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_weight(gpui::FontWeight::BLACK)
                                            .text_color(cx.theme().foreground)
                                            .child("附近的设备"),
                                    )
                                    .child(
                                        h_flex()
                                            .gap(spacing::SM)
                                            .items_center()
                                            // Scan button (wrapped in RotatingWidget)
                                            .child(
                                                div()
                                                    .relative()
                                                    .child(
                                                        RotatingWidget::new(
                                                            render_action_button(
                                                                "send-scan",
                                                                "icons/refresh.svg",
                                                                cx,
                                                                |this, _window, cx| {
                                                                    this.start_discovery_scan(cx);
                                                                },
                                                            ),
                                                        )
                                                        .spinning(scanning && animations)
                                                        .reverse(true)
                                                        .duration(2),
                                                    ),
                                            )
                                            // Manual address button
                                            .child(render_action_button(
                                                "send-manual",
                                                "icons/target.svg",
                                                cx,
                                                |this, window, cx| {
                                                    if !this.ensure_has_selected_files(window, cx) {
                                                        return;
                                                    }
                                                    this.open_send_to_address_dialog(window, cx);
                                                },
                                            ))
                                            // Favorites button
                                            .child(render_action_button(
                                                "send-favorites",
                                                "icons/heart.svg",
                                                cx,
                                                |this, window, cx| {
                                                    if !this.ensure_has_selected_files(window, cx) {
                                                        return;
                                                    }
                                                    this.open_simple_notice_dialog("收藏夹发送即将接入。", window, cx);
                                                },
                                            ))
                                            // Send mode button
                                            .child(render_action_button(
                                                "send-mode",
                                                "icons/settings.svg",
                                                cx,
                                                |this, window, cx| this.cycle_send_mode(window, cx),
                                            )),
                                    ),
                            ),
                    )
                    // -- Device list or placeholder --
                    .child(
                        if app.send_state.nearby_devices.is_empty() {
                            div()
                                .px(px(15.))
                                .pb(px(10.))
                                .child(DevicePlaceholder)
                        } else {
                            v_flex()
                                .gap(px(10.))
                                .w_full()
                                .children(app.send_state.nearby_devices.iter().map(|device| {
                                    let home_entity = home_entity.clone();
                                    div()
                                        .px(px(15.))
                                        .pb(px(10.))
                                        .child(
                                            DeviceCard::new(device.clone())
                                                .on_select(move |device, window, cx| {
                                                    let device = device.clone();
                                                    home_entity.update(cx, |this, cx| {
                                                        if !this.ensure_has_selected_files(window, cx) {
                                                            return;
                                                        }
                                                        this.send_state.target_device = Some(device);
                                                        if let Some(endpoint) = this
                                                            .send_state
                                                            .target_device
                                                            .as_ref()
                                                            .and_then(|d| this.send_state.nearby_endpoints.get(&d.token))
                                                            .cloned()
                                                        {
                                                            this.execute_send(endpoint.ip, endpoint.port, cx);
                                                        } else {
                                                            this.send_state.target_ip = None;
                                                            this.open_send_to_address_dialog(window, cx);
                                                        }
                                                    });
                                                })
                                        )
                                }))
                        },
                    )
                    // -- Troubleshoot button --
                    .child(
                        div()
                            .w_full()
                            .py(px(10.))
                            .flex()
                            .justify_center()
                            .items_center()
                            .child(
                                Button::new("troubleshoot")
                                    .ghost()
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.open_simple_notice_dialog("请确认目标设备与本机在同一 Wi-Fi 网络。", window, cx);
                                    }))
                                    .child("故障排查"),
                            ),
                    )
                    .child(div().h(px(10.)))
                    // -- OpacitySlideshow hints --
                    .child(
                        div()
                            .px(px(15.))
                            .child(
                                OpacitySlideshow::new(vec![
                                    "选择文件并选择附近设备即可发送".to_string(),
                                    "请确保两台设备在同一网络中".to_string(),
                                ])
                                .duration_millis(6000)
                                .switch_duration_millis(300)
                                .running(animations),
                            ),
                    )
                    .child(div().h(px(20.))),
                ),
            ),
    )
    .into_any_element()
}
