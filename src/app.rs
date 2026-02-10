use crate::state::{app_state::AppState, device_state::DeviceState, transfer_state::TransferState};
use crate::ui::pages::{
    ColorMode, QuickSaveMode, ReceivePageState, SendMode, SendPageState, SettingsPageState,
    ThemeMode,
};
use gpui::{div, prelude::*, px, AnyElement, Context, Entity, IntoElement, Window};
use gpui_component::{
    h_flex,
    tab::{Tab, TabBar},
    v_flex, ActiveTheme as _, Icon, Root, Sizable as _, StyledExt as _,
};
use uuid::Uuid;

/// Main application using new modular structure
pub struct NearSendApp {
    app_state: Entity<AppState>,
    device_state: Entity<DeviceState>,
    transfer_state: Entity<TransferState>,
    current_tab: TabType,
    services_started: bool,
    // Page states
    receive_state: ReceivePageState,
    send_state: SendPageState,
    settings_state: SettingsPageState,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TabType {
    Receive,
    Send,
    Settings,
}

impl NearSendApp {
    /// Create a new NearSendApp instance
    pub fn new(
        app_state: Entity<AppState>,
        device_state: Entity<DeviceState>,
        transfer_state: Entity<TransferState>,
    ) -> Self {
        Self {
            app_state,
            device_state,
            transfer_state,
            current_tab: TabType::Receive,
            services_started: false,
            receive_state: ReceivePageState::default(),
            send_state: SendPageState::default(),
            settings_state: SettingsPageState::default(),
        }
    }
}

impl gpui::Render for NearSendApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        // Start services only once using GPUI async context (OpenHarmony compatible)
        if !self.services_started {
            self.services_started = true;
            // TODO: Start discovery and server services
        }

        // Main UI container with full flex layout (mobile-first design)
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                // Content area
                div()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .child(match self.current_tab {
                        TabType::Receive => self.render_receive_content(window, cx),
                        TabType::Send => self.render_send_content(window, cx),
                        TabType::Settings => self.render_settings_content(window, cx),
                    }),
            )
            .child(
                // Bottom Navigation Bar: no divider with content, selected = theme color (no bg), unselected = gray
                div()
                    .w_full()
                    .bg(cx.theme().background)
                    .py(px(6.))
                    .child(self.render_bottom_nav(cx)),
            )
            .children(sheet_layer)
            .children(dialog_layer)
            .children(notification_layer)
    }
}

impl NearSendApp {
    fn render_bottom_nav(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let items: [(TabType, &'static str, &'static str); 3] = [
            (TabType::Receive, "接收", "icons/wifi.svg"),
            (TabType::Send, "发送", "icons/send-horizontal.svg"),
            (TabType::Settings, "设置", "icons/settings.svg"),
        ];

        h_flex()
            .w_full()
            .items_center()
            .children(items.iter().map(|(tab, label, icon_path)| {
                div().flex_1().child(self.render_bottom_nav_item(*tab, label, *icon_path, cx))
            }))
            .into_any_element()
    }

    fn render_bottom_nav_item(
        &mut self,
        tab: TabType,
        label: &'static str,
        icon_path: &'static str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = self.current_tab == tab;
        let tab_id = format!("tab-{}", label.to_lowercase());
        let text_color = if selected {
            cx.theme().primary
        } else {
            cx.theme().muted_foreground
        };

        let icon_el = Icon::default()
            .path(icon_path)
            .text_color(text_color)
            .with_size(gpui_component::Size::Large);

        div()
            .id(tab_id)
            .w_full()
            .h(px(56.))
            .py(px(6.))
            .flex()
            .items_center()
            .justify_center()
            .on_click(cx.listener(move |this, _event, _window, _cx| {
                this.current_tab = tab;
            }))
            .child(
                v_flex()
                    .items_center()
                    .gap(px(2.))
                    .text_color(text_color)
                    .child(icon_el)
                    .child(
                        div()
                            .when(selected, |this| this.text_base())
                            .when(!selected, |this| this.text_sm())
                            .child(label),
                    ),
            )
            .into_any_element()
    }

    /// Render receive page content
    fn render_receive_content(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use crate::ui::components::{
            animated_crossfade::AnimatedCrossFade, animated_opacity::AnimatedOpacity,
            custom_icon_button::CustomIconButton, logo::Logo, rotating_widget::RotatingWidget,
        };
        use crate::ui::theme::{sizing, spacing};
        use gpui_component::scroll::ScrollableElement as _;
        use gpui_component::{
            button::{Button, ButtonVariants as _},
            h_flex,
            tab::{Tab, TabBar},
            v_flex, Size, Sizable as _,
        };

        let show_advanced = self.receive_state.show_advanced;
        let show_history_button = self.receive_state.show_history_button;
        let quick_save_mode = self.receive_state.quick_save_mode;
        let server_alias = self.receive_state.server_alias.clone();
        let server_ips = self.receive_state.server_ips.clone();
        let server_port = self.receive_state.server_port;
        let server_running = self.receive_state.server_running;
        let animations = self.settings_state.animations;

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .relative()
            .child(
                div().flex_1().w_full().overflow_y_scrollbar().child(
                    div().w_full().items_center().child(
                        div()
                            .w_full()
                            .max_w(px(600.)) // ResponsiveListView.defaultMaxWidth
                            .p(px(30.)) // Padding 30 matching localsend
                            .child(
                                v_flex()
                                    .w_full()
                                    .gap(spacing::XL)
                                    .items_center()
                                    .child(
                                        v_flex()
                                            .flex_1()
                                            .items_center()
                                            .justify_center()
                                            .gap(spacing::MD)
                                            .child(
                                                // Logo with rotation animation (when server running and animations enabled)
                                                RotatingWidget::new(
                                                    Logo::new().size(200.).with_text(false),
                                                )
                                                .spinning(
                                                    server_running
                                                        && animations
                                                        && self.current_tab == TabType::Receive,
                                                )
                                                .duration(15), // 15 seconds matching localsend
                                            )
                                            .child(
                                                // Alias with FittedBox (scaleDown) matching localsend
                                                div()
                                                    .max_w(px(520.))
                                                    .text_3xl()
                                                    .font_bold()
                                                    .text_color(cx.theme().foreground)
                                                    .text_center()
                                                    .child(server_alias.clone()),
                                            )
                                            .child(
                                                // IP display with InitialFadeTransition delay
                                                div()
                                                    .max_w(px(520.))
                                                    .text_xl()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .text_center()
                                                    .child(
                                                        if server_running && !server_ips.is_empty()
                                                        {
                                                            // Display multiple IPs like localsend: #ip1 #ip2
                                                            server_ips
                                                                .iter()
                                                                .map(|ip| format!("#{}", ip))
                                                                .collect::<Vec<_>>()
                                                                .join(" ")
                                                        } else {
                                                            "Offline".to_string()
                                                        },
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // Quick Save section with padding top: 10
                                        div().pt(px(10.)).child(
                                            v_flex()
                                                .gap(px(10.))
                                                .items_center()
                                                .child(
                                                    div()
                                                        .text_base()
                                                        .text_color(cx.theme().foreground)
                                                        .child("Quick Save"),
                                                )
                                                .child(
                                                    TabBar::new("quick-save")
                                                        .w_full()
                                                        .segmented()
                                                        .with_size(Size::Small)
                                                        .selected_index(match quick_save_mode {
                                                            QuickSaveMode::Off => 0,
                                                            QuickSaveMode::Favorites => 1,
                                                            QuickSaveMode::On => 2,
                                                        })
                                                        .on_click(cx.listener(
                                                            |this, index, _window, _cx| {
                                                                this.receive_state
                                                                    .quick_save_mode = match *index
                                                                {
                                                                    0 => QuickSaveMode::Off,
                                                                    1 => QuickSaveMode::Favorites,
                                                                    2 => QuickSaveMode::On,
                                                                    _ => QuickSaveMode::Off,
                                                                };
                                                            },
                                                        ))
                                                        .children([
                                                            Tab::new().flex_1().label("Off"),
                                                            Tab::new().flex_1().label("Favorites"),
                                                            Tab::new().flex_1().label("On"),
                                                        ]),
                                                ),
                                        ),
                                    )
                                    .child(div().h(px(15.))),
                            ),
                    ),
                ),
            )
            .child(
                // CornerButtons matching localsend - padding: 20, alignment: topRight
                div().absolute().top(px(20.)).right(px(20.)).child(
                    h_flex()
                        .gap(spacing::SM)
                        .justify_end()
                        .child(div().when(!show_advanced, |this| {
                            this.child(
                                AnimatedOpacity::new(
                                    "receive-history",
                                    if show_history_button { 1.0 } else { 0.0 },
                                    CustomIconButton::new("history", "📜").on_click(|_window, _cx| {
                                        log::info!("History clicked");
                                    }),
                                )
                                .duration_ms(200),
                            )
                        }))
                        .child(
                            Button::new("info")
                                .ghost()
                                .rounded_full()
                                .p(px(8.))
                                .on_click(cx.listener(|this, _event, _window, _cx| {
                                    this.receive_state.show_advanced =
                                        !this.receive_state.show_advanced;
                                }))
                                .child("ℹ️"),
                        ),
                ),
            )
            .child(
                // InfoBox with AnimatedCrossFade matching localsend
                div()
                    .absolute()
                    .top(px(15.))
                    .right(px(15.))
                    .child(
                        AnimatedCrossFade::new("receive-info", show_advanced)
                            .duration_ms(200)
                            .first(div())
                            .second(
                                div()
                                    .bg(cx.theme().secondary)
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .p(px(15.))
                                    .shadow_lg()
                                    .child(
                                        // Table layout matching localsend exactly
                                        v_flex()
                                            .gap(spacing::SM)
                                            .child(
                                                // Alias row - TableRow equivalent
                                                h_flex()
                                                    .gap(px(10.))
                                                    .items_start()
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child("Alias:"),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(cx.theme().foreground)
                                                            .pr(px(30.))
                                                            .child(server_alias.clone()),
                                                    ),
                                            )
                                            .child(
                                                // IP row - TableRow with Column for multiple IPs
                                                h_flex()
                                                    .gap(px(10.))
                                                    .items_start()
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child("IP:"),
                                                    )
                                                    .child(if server_ips.is_empty() {
                                                        div()
                                                            .text_sm()
                                                            .text_color(cx.theme().foreground)
                                                            .child("Unknown")
                                                    } else {
                                                        v_flex()
                                                            .gap(px(2.))
                                                            .items_start()
                                                            .children(server_ips.iter().map(|ip| {
                                                                div()
                                                                    .text_sm()
                                                                    .text_color(cx.theme().foreground)
                                                                    .child(ip.clone())
                                                            }))
                                                    }),
                                            )
                                            .child(
                                                // Port row - TableRow
                                                h_flex()
                                                    .gap(px(10.))
                                                    .items_start()
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child("Port:"),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(cx.theme().foreground)
                                                            .child(server_port.to_string()),
                                                    ),
                                            ),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }

    /// Render send page content
    fn render_send_content(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        use crate::ui::components::{
            big_button::BigButton, custom_icon_button::CustomIconButton,
            device_card::DeviceCard, device_placeholder::DevicePlaceholder,
            opacity_slideshow::OpacitySlideshow, responsive_wrap_view::ResponsiveWrapView,
            rotating_widget::RotatingWidget,
        };
        use crate::ui::theme::{sizing, spacing};
        use crate::ui::utils::format_file_size;
        use gpui_component::scroll::ScrollableElement as _;
        use gpui_component::{
            button::{Button, ButtonVariants as _},
            h_flex, v_flex,
        };

        let selected_files = self.send_state.selected_files.clone();
        let has_files = !selected_files.is_empty();
        let scanning = self.send_state.scanning;
        let total_size = self.send_state.selected_files_total_size;
        let animations = self.settings_state.animations;
        let local_ip_count = self.send_state.local_ips.len();

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
                            .pt(px(20.)) // SizedBox(height: 20) matching localsend
                            .child(
                                v_flex()
                                    .w_full()
                                    .gap(spacing::MD)
                                    .child(
                                        div()
                                            .px(px(15.)) // _horizontalPadding matching localsend
                                            .child(
                                                div()
                                                    .text_lg()
                                                    .font_semibold()
                                                    .text_color(cx.theme().foreground)
                                                    .child("Select Files"),
                                            ),
                                    )
                                    .when(!has_files, |this| {
                                        this.child(
                                            ResponsiveWrapView::new(BigButton::MOBILE_WIDTH)
                                                .outer_horizontal_padding(15.0)
                                                .outer_vertical_padding(10.0)
                                                .child_padding(10.0)
                                                .child(
                                                    BigButton::new("📷", "Photos")
                                                        .filled(false)
                                                        .on_tap(|_window, _cx| {
                                                            log::info!("Photos clicked");
                                                        }),
                                                )
                                                .child(
                                                    BigButton::new("🎥", "Videos")
                                                        .filled(false)
                                                        .on_tap(|_window, _cx| {
                                                            log::info!("Videos clicked");
                                                        }),
                                                )
                                                .child(
                                                    BigButton::new("📄", "Files")
                                                        .filled(false)
                                                        .on_tap(|_window, _cx| {
                                                            log::info!("Files clicked");
                                                        }),
                                                ),
                                        )
                                    })
                                    .when(has_files, |this| {
                                        this.child(
                                            // Card with margin matching localsend: bottom: 10, left/right: _horizontalPadding
                                            div()
                                                .mx(px(15.))
                                                .mb(px(10.))
                                                .bg(cx.theme().secondary)
                                                .border_1()
                                                .border_color(cx.theme().border)
                                                .rounded_lg()
                                                .pl(px(15.)) // padding left: 15
                                                .pt(px(5.))  // padding top: 5
                                                .pb(px(15.)) // padding bottom: 15
                                                .child(
                                                    v_flex()
                                                        .gap(spacing::MD)
                                                        .child(
                                                            h_flex()
                                                                .justify_between()
                                                                .items_center()
                                                                .child(
                                                                    div()
                                                                        .text_lg()
                                                                        .font_semibold()
                                                                        .text_color(cx.theme().foreground)
                                                                        .child("Select Files"),
                                                                )
                                                                .child(
                                                                    Button::new("clear")
                                                                        .ghost()
                                                                        .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                            this.send_state.selected_files.clear();
                                                                        }))
                                                                        .child("✕"),
                                                                ),
                                                        )
                                                        .child(
                                                            v_flex()
                                                                .gap(spacing::XS)
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .text_color(cx.theme().foreground)
                                                                        .child(format!("{} files", selected_files.len())),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .text_color(cx.theme().muted_foreground)
                                                                        .child(format_file_size(total_size)),
                                                                ),
                                                        )
                                                        .child(
                                                            // File thumbnails horizontal scroll matching localsend
                                                            div()
                                                                .h(px(80.)) // defaultThumbnailSize
                                                                .overflow_x_scrollbar()
                                                                .child(
                                                                    h_flex()
                                                                        .gap(px(10.))
                                                                        .children(selected_files.iter().enumerate().map(|(i, _path)| {
                                                                            div()
                                                                                .pr(px(10.))
                                                                                .child(
                                                                                    div()
                                                                                        .w(px(80.))
                                                                                        .h(px(80.))
                                                                                        .bg(cx.theme().muted)
                                                                                        .rounded_md()
                                                                                        .items_center()
                                                                                        .justify_center()
                                                                                        .child(format!("File {}", i + 1))
                                                                                )
                                                                        })),
                                                                ),
                                                        )
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
                                                                        .child("Edit"),
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
                                                                                .child("➕")
                                                                                .child("Add"),
                                                                        ),
                                                                ),
                                                        ),
                                                ),
                                        )
                                    })
                                    .child(
                                        // Row with Nearby Devices title and buttons matching localsend
                                        div()
                                            .px(px(15.)) // _horizontalPadding
                                            .py(px(10.)) // padding vertical: 10
                                            .child(
                                                h_flex()
                                                    .justify_between()
                                                    .items_center()
                                                    .child(
                                                        div()
                                                            .text_lg()
                                                            .font_semibold()
                                                            .text_color(cx.theme().foreground)
                                                            .child("Nearby Devices"),
                                                    )
                                                    .child(
                                                        h_flex()
                                                            .gap(px(10.))
                                                            .child(
                                                                // Scan button with RotatingWidget matching localsend
                                                                div()
                                                                    .relative()
                                                                    .child(
                                                                        RotatingWidget::new(
                                                                            Button::new("scan")
                                                                                .ghost()
                                                                                .rounded_full()
                                                                                .p(px(8.))
                                                                                .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                    if this.send_state.local_ips.len() > 1 {
                                                                                        this.send_state.show_scan_menu = !this.send_state.show_scan_menu;
                                                                                        this.send_state.show_send_mode_menu = false;
                                                                                    } else {
                                                                                        this.send_state.scanning = true;
                                                                                        this.send_state.nearby_devices.clear();
                                                                                    }
                                                                                }))
                                                                                .child("🔄")
                                                                        )
                                                                        .spinning(scanning && animations)
                                                                        .reverse(true)
                                                                        .duration(2), // 2 seconds matching localsend
                                                                    )
                                                                    .when(self.send_state.show_scan_menu && self.send_state.local_ips.len() > 1, |this| {
                                                                        this.child(
                                                                            div()
                                                                                .absolute()
                                                                                .top(px(40.))
                                                                                .right(px(0.))
                                                                                .bg(cx.theme().secondary)
                                                                                .border_1()
                                                                                .border_color(cx.theme().border)
                                                                                .rounded_lg()
                                                                                .overflow_hidden()
                                                                                .shadow_lg()
                                                                                .py(px(6.))
                                                                                .child(
                                                                                    v_flex()
                                                                                        .children(self.send_state.local_ips.iter().enumerate().flat_map(|(index, ip)| {
                                                                                            let mut items = Vec::new();
                                                                                            items.push(
                                                                                                h_flex()
                                                                                                    .gap(px(10.))
                                                                                                    .items_center()
                                                                                                    .px(px(12.))
                                                                                                    .py(px(6.))
                                                                                                    .child(
                                                                                                        RotatingWidget::new(
                                                                                                            div().text_sm().child("🔄")
                                                                                                        )
                                                                                                        .spinning(false) // TODO: Check if this IP is scanning
                                                                                                        .reverse(true)
                                                                                                        .duration(2),
                                                                                                    )
                                                                                                    .child(
                                                                                                        Button::new(format!("scan-{}", ip))
                                                                                                            .ghost()
                                                                                                            .w_full()
                                                                                                            .on_click(cx.listener(move |this, _event, _window, _cx| {
                                                                                                                this.send_state.show_scan_menu = false;
                                                                                                                this.send_state.scanning = true;
                                                                                                            }))
                                                                                                            .child(ip.clone())
                                                                                                    )
                                                                                                    .into_any_element(),
                                                                                            );
                                                                                            if index + 1 < local_ip_count {
                                                                                                items.push(
                                                                                                    div()
                                                                                                        .h(px(1.))
                                                                                                        .bg(cx.theme().border)
                                                                                                        .into_any_element(),
                                                                                                );
                                                                                            }
                                                                                            items
                                                                                        })),
                                                                                ),
                                                                        )
                                                                    }),
                                                            )
                                                            .child(
                                                                CustomIconButton::new("manual", "📍").on_click(|_window, _cx| {
                                                                    log::info!("Manual address clicked");
                                                                }),
                                                            )
                                                            .child(
                                                                CustomIconButton::new("favorites", "⭐").on_click(|_window, _cx| {
                                                                    log::info!("Favorites clicked");
                                                                }),
                                                            )
                                                            .child(
                                                                // Send mode button with popup menu matching localsend
                                                                div()
                                                                    .relative()
                                                                    .child(
                                                                        Button::new("send-mode")
                                                                            .ghost()
                                                                            .rounded_full()
                                                                            .p(px(8.))
                                                                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                this.send_state.show_send_mode_menu = !this.send_state.show_send_mode_menu;
                                                                                this.send_state.show_scan_menu = false;
                                                                            }))
                                                                            .child("⚙️"),
                                                                    )
                                                                    .when(self.send_state.show_send_mode_menu, |this| {
                                                                        this.child(
                                                                            div()
                                                                                .absolute()
                                                                                .top(px(40.))
                                                                                .right(px(0.))
                                                                                .bg(cx.theme().secondary)
                                                                                .border_1()
                                                                                .border_color(cx.theme().border)
                                                                                .rounded_lg()
                                                                                .overflow_hidden()
                                                                                .shadow_lg()
                                                                                .py(px(6.))
                                                                                .child(
                                                                                    v_flex()
                                                                                        .child(
                                                                                            // Single mode with check icon
                                                                                            h_flex()
                                                                                                .gap(px(10.))
                                                                                                .items_center()
                                                                                                .px(px(12.))
                                                                                                .py(px(6.))
                                                                                                .child(
                                                                                                    div()
                                                                                                        .when(self.send_state.send_mode == SendMode::Single, |this| {
                                                                                                            this.child(div().text_sm().child("✓"))
                                                                                                        })
                                                                                                        .when(self.send_state.send_mode != SendMode::Single, |this| {
                                                                                                            this.w(px(20.)).h(px(20.)) // Maintain size
                                                                                                        }),
                                                                                                )
                                                                                                .child(
                                                                                                    Button::new("mode-single")
                                                                                                        .ghost()
                                                                                                        .w_full()
                                                                                                        .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                                            this.send_state.show_send_mode_menu = false;
                                                                                                            this.send_state.send_mode = SendMode::Single;
                                                                                                        }))
                                                                                                        .child("Single"),
                                                                                                ),
                                                                                        )
                                                                                        .child(
                                                                                            div()
                                                                                                .h(px(1.))
                                                                                                .bg(cx.theme().border),
                                                                                        )
                                                                                        .child(
                                                                                            // Multiple mode with check icon
                                                                                            h_flex()
                                                                                                .gap(px(10.))
                                                                                                .items_center()
                                                                                                .px(px(12.))
                                                                                                .py(px(6.))
                                                                                                .child(
                                                                                                    div()
                                                                                                        .when(self.send_state.send_mode == SendMode::Multiple, |this| {
                                                                                                            this.child(div().text_sm().child("✓"))
                                                                                                        })
                                                                                                        .when(self.send_state.send_mode != SendMode::Multiple, |this| {
                                                                                                            this.w(px(20.)).h(px(20.)) // Maintain size
                                                                                                        }),
                                                                                                )
                                                                                                .child(
                                                                                                    Button::new("mode-multiple")
                                                                                                        .ghost()
                                                                                                        .w_full()
                                                                                                        .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                                            this.send_state.show_send_mode_menu = false;
                                                                                                            this.send_state.send_mode = SendMode::Multiple;
                                                                                                        }))
                                                                                                        .child("Multiple"),
                                                                                                ),
                                                                                        )
                                                                                        .child(
                                                                                            div()
                                                                                                .h(px(1.))
                                                                                                .bg(cx.theme().border),
                                                                                        )
                                                                                        .child(
                                                                                            // Link mode (no check)
                                                                                            h_flex()
                                                                                                .gap(px(10.))
                                                                                                .items_center()
                                                                                                .px(px(12.))
                                                                                                .py(px(6.))
                                                                                                .child(
                                                                                                    div().w(px(20.)).h(px(20.)), // Maintain size
                                                                                                )
                                                                                                .child(
                                                                                                    Button::new("mode-link")
                                                                                                        .ghost()
                                                                                                        .w_full()
                                                                                                        .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                                            this.send_state.show_send_mode_menu = false;
                                                                                                            this.send_state.send_mode = SendMode::Link;
                                                                                                        }))
                                                                                                        .child("Link"),
                                                                                                ),
                                                                                        )
                                                                                        .child(
                                                                                            div()
                                                                                                .h(px(1.))
                                                                                                .bg(cx.theme().border),
                                                                                        )
                                                                                        .child(
                                                                                            // Help option
                                                                                            h_flex()
                                                                                                .gap(px(10.))
                                                                                                .items_center()
                                                                                                .px(px(12.))
                                                                                                .py(px(6.))
                                                                                                .child(
                                                                                                    div().text_sm().child("❓"),
                                                                                                )
                                                                                                .child(
                                                                                                    Button::new("mode-help")
                                                                                                        .ghost()
                                                                                                        .w_full()
                                                                                                        .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                                            this.send_state.show_send_mode_menu = false;
                                                                                                            log::info!("Send mode help clicked");
                                                                                                        }))
                                                                                                        .child("Help"),
                                                                                                ),
                                                                                        ),
                                                                                ),
                                                                        )
                                                                    }),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // Device list matching localsend
                                        if self.send_state.nearby_devices.is_empty() {
                                            // DevicePlaceholder with opacity 0.3 matching localsend
                                            div()
                                                .px(px(15.)) // _horizontalPadding
                                                .pb(px(10.))
                                                .child(
                                                    DevicePlaceholder
                                                )
                                        } else {
                                            v_flex()
                                                .gap(px(10.))
                                                .w_full()
                                                .children(self.send_state.nearby_devices.iter().map(|device| {
                                                    div()
                                                        .px(px(15.)) // _horizontalPadding
                                                        .pb(px(10.))
                                                        .child(
                                                            DeviceCard::new(device.clone())
                                                                .on_select(|device, _window, _cx| {
                                                                    log::info!("Device selected: {}", device.alias);
                                                                })
                                                        )
                                                }))
                                        },
                                    )
                                    .child(
                                        // Troubleshoot button matching localsend - TextButton style
                                        div()
                                            .w_full()
                                            .py(px(10.))
                                            .text_center()
                                            .child(
                                                Button::new("troubleshoot")
                                                    .ghost()
                                                    .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                        log::info!("Troubleshoot clicked");
                                                    }))
                                                    .child("Troubleshoot"),
                                            ),
                                    )
                                    .child(div().h(px(20.))) // SizedBox(height: 20)
                                    .child(
                                        // OpacitySlideshow matching localsend
                                        div()
                                            .px(px(15.)) // _horizontalPadding
                                            .child(
                                                OpacitySlideshow::new(vec![
                                                    "Select files and choose a nearby device to send".to_string(),
                                                    "Ensure both devices are on the same network".to_string(),
                                                ])
                                                .duration_millis(6000)
                                                .switch_duration_millis(300)
                                                .running(animations),
                                            ),
                                    )
                                    .child(div().h(px(50.))), // SizedBox(height: 50)
                                ),
                    ),
            )
            .into_any_element()
    }

    /// Render settings page content
    fn render_settings_content(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use crate::ui::components::{logo::Logo, switch::Switch};
        use crate::ui::theme::{sizing, spacing};
        use gpui_component::scroll::ScrollableElement as _;
        use gpui_component::{
            button::{Button, ButtonVariants as _},
            h_flex, v_flex,
        };

        let advanced = self.settings_state.advanced;
        let animations = self.settings_state.animations;
        let quick_save = self.settings_state.quick_save;
        let quick_save_favorites = self.settings_state.quick_save_favorites;
        let require_pin = self.settings_state.require_pin;
        let auto_finish = self.settings_state.auto_finish;
        let save_to_history = self.settings_state.save_to_history;
        let save_to_gallery = self.settings_state.save_to_gallery;
        let share_via_link = self.settings_state.share_via_link_auto_accept;
        let server_alias = self.settings_state.server_alias.clone();
        let server_port = self.settings_state.server_port;
        let server_running = self.settings_state.server_running;
        let theme_mode = self.settings_state.theme_mode;
        let color_mode = self.settings_state.color_mode;
        let language = self.settings_state.language.clone();
        let destination = self.settings_state.destination.clone();
        let encryption = self.settings_state.encryption;
        let network_filtered = self.settings_state.network_filtered;

        div()
            .size_full()
            .w_full()
            .bg(cx.theme().background)
            .overflow_y_scrollbar()
            .child(
                v_flex()
                    .w_full()
                    .px(px(15.)) // horizontal: 15 matching localsend
                    .py(px(40.)) // vertical: 40 matching localsend
                    .gap(spacing::LG)
                    .child(div().h(px(30.))) // SizedBox(height: 30 + padding.top) - simplified
                    .child(
                        // General section - _SettingsSection matching localsend
                        div()
                            .bg(cx.theme().secondary)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .pl(px(15.)) // padding left: 15
                            .pr(px(15.)) // padding right: 15
                            .pt(px(15.)) // padding top: 15
                            .pb(px(15.)) // padding bottom: 15 (default)
                            .child(
                                v_flex()
                                    .gap(px(10.)) // SizedBox(height: 10) after title
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_semibold()
                                            .text_color(cx.theme().foreground)
                                            .child("General"),
                                    )
                                    .child(
                                        // _SettingsEntry for Brightness - Row with Expanded label, SizedBox(width: 10), SizedBox(width: 150) child
                                        div()
                                            .pb(px(15.)) // padding bottom: 15
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(cx.theme().foreground)
                                                            .flex_1()
                                                            .child("Brightness"),
                                                    )
                                                    .child(div().w(px(10.))) // SizedBox(width: 10)
                                                    .child(
                                                        div()
                                                            .w(px(150.)) // SizedBox(width: 150)
                                                            .child(
                                                                // CustomDropdownButton equivalent
                                                                div()
                                                                    .bg(cx.theme().muted)
                                                                    .rounded_md()
                                                                    .p(px(8.))
                                                                    .child(
                                                                        h_flex()
                                                                            .justify_center()
                                                                            .gap(px(4.))
                                                                            .child(
                                                                                Button::new("theme-system")
                                                                                    .with_variant(if theme_mode == ThemeMode::System {
                                                                                        gpui_component::button::ButtonVariant::Primary
                                                                                    } else {
                                                                                        gpui_component::button::ButtonVariant::Ghost
                                                                                    })
                                                                                    .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                        this.settings_state.theme_mode = ThemeMode::System;
                                                                                    }))
                                                                                    .child("System"),
                                                                            )
                                                                            .child(
                                                                                Button::new("theme-light")
                                                                                    .with_variant(if theme_mode == ThemeMode::Light {
                                                                                        gpui_component::button::ButtonVariant::Primary
                                                                                    } else {
                                                                                        gpui_component::button::ButtonVariant::Ghost
                                                                                    })
                                                                                    .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                        this.settings_state.theme_mode = ThemeMode::Light;
                                                                                    }))
                                                                                    .child("Light"),
                                                                            )
                                                                            .child(
                                                                                Button::new("theme-dark")
                                                                                    .with_variant(if theme_mode == ThemeMode::Dark {
                                                                                        gpui_component::button::ButtonVariant::Primary
                                                                                    } else {
                                                                                        gpui_component::button::ButtonVariant::Ghost
                                                                                    })
                                                                                    .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                        this.settings_state.theme_mode = ThemeMode::Dark;
                                                                                    }))
                                                                                    .child("Dark"),
                                                                            ),
                                                                    ),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // _SettingsEntry for Color
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
                                                            .child("Color"),
                                                    )
                                                    .child(div().w(px(10.)))
                                                    .child(
                                                        div()
                                                            .w(px(150.))
                                                            .child(
                                                                div()
                                                                    .bg(cx.theme().muted)
                                                                    .rounded_md()
                                                                    .p(px(8.))
                                                                    .child(
                                                                        h_flex()
                                                                            .justify_center()
                                                                            .gap(px(4.))
                                                                            .child(
                                                                                Button::new("color-system")
                                                                                    .with_variant(if color_mode == ColorMode::System {
                                                                                        gpui_component::button::ButtonVariant::Primary
                                                                                    } else {
                                                                                        gpui_component::button::ButtonVariant::Ghost
                                                                                    })
                                                                                    .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                        this.settings_state.color_mode = ColorMode::System;
                                                                                    }))
                                                                                    .child("System"),
                                                                            )
                                                                            .child(
                                                                                Button::new("color-localsend")
                                                                                    .with_variant(if color_mode == ColorMode::LocalSend {
                                                                                        gpui_component::button::ButtonVariant::Primary
                                                                                    } else {
                                                                                        gpui_component::button::ButtonVariant::Ghost
                                                                                    })
                                                                                    .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                        this.settings_state.color_mode = ColorMode::LocalSend;
                                                                                    }))
                                                                                    .child("NearSend"),
                                                                            )
                                                                            .child(
                                                                                Button::new("color-oled")
                                                                                    .with_variant(if color_mode == ColorMode::Oled {
                                                                                        gpui_component::button::ButtonVariant::Primary
                                                                                    } else {
                                                                                        gpui_component::button::ButtonVariant::Ghost
                                                                                    })
                                                                                    .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                        this.settings_state.color_mode = ColorMode::Oled;
                                                                                    }))
                                                                                    .child("OLED"),
                                                                            ),
                                                                    ),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // _ButtonEntry for Language matching localsend
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
                                                            .child("Language"),
                                                    )
                                                    .child(div().w(px(10.)))
                                                    .child(
                                                        div()
                                                            .w(px(150.))
                                                            .child(
                                                                Button::new("language-btn")
                                                                    .with_variant(gpui_component::button::ButtonVariant::Secondary)
                                                                    .outline()
                                                                    .w_full()
                                                                    .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                                        log::info!("Language clicked");
                                                                    }))
                                                                    .child(language.clone()),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // _BooleanEntry matching localsend - Stack + Container + Switch centered
                                        div()
                                            .pb(px(15.)) // padding bottom: 15
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(cx.theme().foreground)
                                                            .flex_1()
                                                            .child("Animations"),
                                                    )
                                                    .child(div().w(px(10.))) // SizedBox(width: 10)
                                                    .child(
                                                        div()
                                                            .w(px(150.)) // SizedBox(width: 150)
                                                            .h(px(50.)) // height: 50
                                                            .relative()
                                                            .child(
                                                                div()
                                                                    .absolute()
                                                                    .inset_0()
                                                                    .bg(cx.theme().muted)
                                                                    .rounded_md(),
                                                            )
                                                            .child(
                                                                div()
                                                                    .absolute()
                                                                    .inset_0()
                                                                    .items_center()
                                                                    .justify_center()
                                                                    .child(
                                                                        Button::new("toggle-animations")
                                                                            .ghost()
                                                                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                this.settings_state.animations = !this.settings_state.animations;
                                                                            }))
                                                                            .child(Switch::new(animations)),
                                                                    ),
                                                            ),
                                                    ),
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        // Receive section - _SettingsSection matching localsend
                        div()
                            .bg(cx.theme().secondary)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .pl(px(15.)) // padding left: 15
                            .pr(px(15.)) // padding right: 15
                            .pt(px(15.)) // padding top: 15
                            .pb(px(15.)) // padding bottom: 15
                            .child(
                                v_flex()
                                    .gap(px(10.)) // SizedBox(height: 10) after title
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_semibold()
                                            .text_color(cx.theme().foreground)
                                            .child("Receive"),
                                    )
                                    .child(
                                        // _BooleanEntry for Quick Save
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
                                                            .child("Quick Save"),
                                                    )
                                                    .child(div().w(px(10.)))
                                                    .child(
                                                        div()
                                                            .w(px(150.))
                                                            .h(px(50.))
                                                            .relative()
                                                            .child(
                                                                div()
                                                                    .absolute()
                                                                    .inset_0()
                                                                    .bg(cx.theme().muted)
                                                                    .rounded_md(),
                                                            )
                                                            .child(
                                                                div()
                                                                    .absolute()
                                                                    .inset_0()
                                                                    .items_center()
                                                                    .justify_center()
                                                                    .child(
                                                                        Button::new("toggle-quick-save")
                                                                            .ghost()
                                                                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                this.settings_state.quick_save = !this.settings_state.quick_save;
                                                                            }))
                                                                            .child(Switch::new(quick_save)),
                                                                    ),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // _BooleanEntry for Quick Save from Favorites
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
                                                            .child("Quick Save from Favorites"),
                                                    )
                                                    .child(div().w(px(10.)))
                                                    .child(
                                                        div()
                                                            .w(px(150.))
                                                            .h(px(50.))
                                                            .relative()
                                                            .child(
                                                                div()
                                                                    .absolute()
                                                                    .inset_0()
                                                                    .bg(cx.theme().muted)
                                                                    .rounded_md(),
                                                            )
                                                            .child(
                                                                div()
                                                                    .absolute()
                                                                    .inset_0()
                                                                    .items_center()
                                                                    .justify_center()
                                                                    .child(
                                                                        Button::new("toggle-quick-save-favorites")
                                                                            .ghost()
                                                                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                this.settings_state.quick_save_favorites = !this.settings_state.quick_save_favorites;
                                                                            }))
                                                                            .child(Switch::new(quick_save_favorites)),
                                                                    ),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // _BooleanEntry for Require PIN
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
                                                            .child("Require PIN"),
                                                    )
                                                    .child(div().w(px(10.)))
                                                    .child(
                                                        div()
                                                            .w(px(150.))
                                                            .h(px(50.))
                                                            .relative()
                                                            .child(
                                                                div()
                                                                    .absolute()
                                                                    .inset_0()
                                                                    .bg(cx.theme().muted)
                                                                    .rounded_md(),
                                                            )
                                                            .child(
                                                                div()
                                                                    .absolute()
                                                                    .inset_0()
                                                                    .items_center()
                                                                    .justify_center()
                                                                    .child(
                                                                        Button::new("toggle-require-pin")
                                                                            .ghost()
                                                                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                this.settings_state.require_pin = !this.settings_state.require_pin;
                                                                            }))
                                                                            .child(Switch::new(require_pin)),
                                                                    ),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // _ButtonEntry for Destination matching localsend
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
                                                            .child("Destination"),
                                                    )
                                                    .child(div().w(px(10.)))
                                                    .child(
                                                        div()
                                                            .w(px(150.))
                                                            .child(
                                                                Button::new("destination")
                                                                    .with_variant(gpui_component::button::ButtonVariant::Secondary)
                                                                    .outline()
                                                                    .w_full()
                                                                    .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                        this.settings_state.destination = Some("Downloads".to_string());
                                                                    }))
                                                                    .child(destination.clone().unwrap_or_else(|| "Downloads".to_string())),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // _BooleanEntry for Save to Gallery
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
                                                            .child("Save to Gallery"),
                                                    )
                                                    .child(div().w(px(10.)))
                                                    .child(
                                                        div()
                                                            .w(px(150.))
                                                            .h(px(50.))
                                                            .relative()
                                                            .child(
                                                                div()
                                                                    .absolute()
                                                                    .inset_0()
                                                                    .bg(cx.theme().muted)
                                                                    .rounded_md(),
                                                            )
                                                            .child(
                                                                div()
                                                                    .absolute()
                                                                    .inset_0()
                                                                    .items_center()
                                                                    .justify_center()
                                                                    .child(
                                                                        Button::new("toggle-save-to-gallery")
                                                                            .ghost()
                                                                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                this.settings_state.save_to_gallery = !this.settings_state.save_to_gallery;
                                                                            }))
                                                                            .child(Switch::new(save_to_gallery)),
                                                                    ),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // _BooleanEntry for Auto Finish
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
                                                            .child("Auto Finish"),
                                                    )
                                                    .child(div().w(px(10.)))
                                                    .child(
                                                        div()
                                                            .w(px(150.))
                                                            .h(px(50.))
                                                            .relative()
                                                            .child(
                                                                div()
                                                                    .absolute()
                                                                    .inset_0()
                                                                    .bg(cx.theme().muted)
                                                                    .rounded_md(),
                                                            )
                                                            .child(
                                                                div()
                                                                    .absolute()
                                                                    .inset_0()
                                                                    .items_center()
                                                                    .justify_center()
                                                                    .child(
                                                                        Button::new("toggle-auto-finish")
                                                                            .ghost()
                                                                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                this.settings_state.auto_finish = !this.settings_state.auto_finish;
                                                                            }))
                                                                            .child(Switch::new(auto_finish)),
                                                                    ),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // _BooleanEntry for Save to History
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
                                                            .child("Save to History"),
                                                    )
                                                    .child(div().w(px(10.)))
                                                    .child(
                                                        div()
                                                            .w(px(150.))
                                                            .h(px(50.))
                                                            .relative()
                                                            .child(
                                                                div()
                                                                    .absolute()
                                                                    .inset_0()
                                                                    .bg(cx.theme().muted)
                                                                    .rounded_md(),
                                                            )
                                                            .child(
                                                                div()
                                                                    .absolute()
                                                                    .inset_0()
                                                                    .items_center()
                                                                    .justify_center()
                                                                    .child(
                                                                        Button::new("toggle-save-to-history")
                                                                            .ghost()
                                                                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                this.settings_state.save_to_history = !this.settings_state.save_to_history;
                                                                            }))
                                                                            .child(Switch::new(save_to_history)),
                                                                    ),
                                                            ),
                                                    ),
                                            ),
                                    ),
                            ),
                    )
                    .when(advanced, |this| {
                        this.child(
                            // Send section - _SettingsSection matching localsend
                            div()
                                .bg(cx.theme().secondary)
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_lg()
                                .pl(px(15.))
                                .pr(px(15.))
                                .pt(px(15.))
                                .pb(px(15.))
                                .child(
                                    v_flex()
                                        .gap(px(10.))
                                        .child(
                                            div()
                                                .text_lg()
                                                .font_semibold()
                                                .text_color(cx.theme().foreground)
                                                .child("Send"),
                                        )
                                        .child(
                                            // _BooleanEntry for Share via Link Auto Accept
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
                                                                .child("Share via Link Auto Accept"),
                                                        )
                                                        .child(div().w(px(10.)))
                                                        .child(
                                                            div()
                                                                .w(px(150.))
                                                                .h(px(50.))
                                                                .relative()
                                                                .child(
                                                                    div()
                                                                        .absolute()
                                                                        .inset_0()
                                                                        .bg(cx.theme().muted)
                                                                        .rounded_md(),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .absolute()
                                                                        .inset_0()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .child(
                                                                            Button::new("toggle-share-via-link")
                                                                                .ghost()
                                                                                .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                    this.settings_state.share_via_link_auto_accept = !this.settings_state.share_via_link_auto_accept;
                                                                                }))
                                                                                .child(Switch::new(share_via_link)),
                                                                        ),
                                                                ),
                                                        ),
                                                ),
                                        ),
                                ),
                        )
                    })
                    .child(
                        // Network section - _SettingsSection matching localsend
                        div()
                            .bg(cx.theme().secondary)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .pl(px(15.))
                            .pr(px(15.))
                            .pt(px(15.))
                            .pb(px(15.))
                            .child(
                                v_flex()
                                    .gap(px(10.))
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_semibold()
                                            .text_color(cx.theme().foreground)
                                            .child(format!("Server{}", if server_running { "" } else { " (Offline)" })),
                                    )
                                    .child(
                                        // _SettingsEntry for Server control matching localsend
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
                                                            .child(format!("Server{}", if server_running { "" } else { " (Offline)" })),
                                                    )
                                                    .child(div().w(px(10.)))
                                                    .child(
                                                        div()
                                                            .w(px(150.))
                                                            .child(
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
                                                                                    .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                        this.settings_state.server_running = true;
                                                                                    }))
                                                                                    .child(if server_running { "🔄" } else { "▶" }),
                                                                            )
                                                                            .child(
                                                                                Button::new("server-stop")
                                                                                    .ghost()
                                                                                    .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                        if this.settings_state.server_running {
                                                                                            this.settings_state.server_running = false;
                                                                                        }
                                                                                    }))
                                                                                    .child("⏹"),
                                                                            ),
                                                                    ),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // _SettingsEntry for Alias matching localsend (TextFieldWithActions)
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
                                                            .child("Alias"),
                                                    )
                                                    .child(div().w(px(10.)))
                                                    .child(
                                                        div()
                                                            .w(px(150.))
                                                            .child(
                                                                Button::new("alias-input")
                                                                    .with_variant(gpui_component::button::ButtonVariant::Secondary)
                                                                    .outline()
                                                                    .w_full()
                                                                    .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                                        log::info!("Alias input clicked");
                                                                    }))
                                                                    .child(server_alias.clone()),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .when(advanced, |this| {
                                        this.child(
                                            // _SettingsEntry for Port matching localsend (TextFieldTv)
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
                                                                .child("Port"),
                                                        )
                                                        .child(div().w(px(10.)))
                                                        .child(
                                                            div()
                                                                .w(px(150.))
                                                                .child(
                                                                    Button::new("port-input")
                                                                        .with_variant(gpui_component::button::ButtonVariant::Secondary)
                                                                        .outline()
                                                                        .w_full()
                                                                        .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                                            log::info!("Port input clicked");
                                                                        }))
                                                                        .child(server_port.to_string()),
                                                                ),
                                                        ),
                                                ),
                                        )
                                        .child(
                                            // _BooleanEntry for Encryption
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
                                                                .child("Encryption"),
                                                        )
                                                        .child(div().w(px(10.)))
                                                        .child(
                                                            div()
                                                                .w(px(150.))
                                                                .h(px(50.))
                                                                .relative()
                                                                .child(
                                                                    div()
                                                                        .absolute()
                                                                        .inset_0()
                                                                        .bg(cx.theme().muted)
                                                                        .rounded_md(),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .absolute()
                                                                        .inset_0()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .child(
                                                                            Button::new("toggle-encryption")
                                                                                .ghost()
                                                                                .on_click(cx.listener(|this, _event, _window, _cx| {
                                                                                    this.settings_state.encryption = !this.settings_state.encryption;
                                                                                }))
                                                                                .child(Switch::new(encryption)),
                                                                        ),
                                                                ),
                                                        ),
                                                ),
                                        )
                                        .child(
                                            // _ButtonEntry for Network matching localsend
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
                                                                .child("Network"),
                                                        )
                                                        .child(div().w(px(10.)))
                                                        .child(
                                                            div()
                                                                .w(px(150.))
                                                                .child(
                                                                    Button::new("network")
                                                                        .with_variant(gpui_component::button::ButtonVariant::Secondary)
                                                                        .outline()
                                                                        .w_full()
                                                                        .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                                            log::info!("Network clicked");
                                                                        }))
                                                                        .child(if network_filtered { "Filtered" } else { "All" }),
                                                                ),
                                                        ),
                                                ),
                                        )
                                    }),
                            ),
                    )
                    .child(
                        // Other section - _SettingsSection matching localsend (padding bottom: 0)
                        div()
                            .bg(cx.theme().secondary)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .pl(px(15.))
                            .pr(px(15.))
                            .pt(px(15.))
                            .pb(px(0.)) // padding bottom: 0 matching localsend
                            .child(
                                v_flex()
                                    .gap(px(10.))
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_semibold()
                                            .text_color(cx.theme().foreground)
                                            .child("Other"),
                                    )
                                    .child(
                                        // _ButtonEntry for About matching localsend
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
                                                            .child("About"),
                                                    )
                                                    .child(div().w(px(10.)))
                                                    .child(
                                                        div()
                                                            .w(px(150.))
                                                            .child(
                                                                Button::new("about")
                                                                    .with_variant(gpui_component::button::ButtonVariant::Secondary)
                                                                    .outline()
                                                                    .w_full()
                                                                    .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                                        log::info!("About clicked");
                                                                    }))
                                                                    .child("Open"),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // _ButtonEntry for Support/Donate matching localsend
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
                                                            .child("Support"),
                                                    )
                                                    .child(div().w(px(10.)))
                                                    .child(
                                                        div()
                                                            .w(px(150.))
                                                            .child(
                                                                Button::new("donate")
                                                                    .with_variant(gpui_component::button::ButtonVariant::Secondary)
                                                                    .outline()
                                                                    .w_full()
                                                                    .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                                        log::info!("Donate clicked");
                                                                    }))
                                                                    .child("Donate"),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // _ButtonEntry for Privacy Policy matching localsend
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
                                                            .child("Privacy Policy"),
                                                    )
                                                    .child(div().w(px(10.)))
                                                    .child(
                                                        div()
                                                            .w(px(150.))
                                                            .child(
                                                                Button::new("privacy")
                                                                    .with_variant(gpui_component::button::ButtonVariant::Secondary)
                                                                    .outline()
                                                                    .w_full()
                                                                    .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                                        log::info!("Privacy clicked");
                                                                    }))
                                                                    .child("Open"),
                                                            ),
                                                    ),
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        // Advanced Settings toggle - LabeledCheckbox matching localsend (labelFirst: true)
                        h_flex()
                            .justify_end()
                            .w_full()
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap(px(5.)) // SizedBox(width: 5) matching localsend
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().foreground)
                                            .child("Advanced Settings"),
                                    )
                                    .child(
                                        Button::new("toggle-advanced-settings")
                                            .with_variant(if advanced {
                                                gpui_component::button::ButtonVariant::Primary
                                            } else {
                                                gpui_component::button::ButtonVariant::Secondary
                                            })
                                            .outline()
                                            .w(px(20.))
                                            .h(px(20.))
                                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                                this.settings_state.advanced = !this.settings_state.advanced;
                                            }))
                                            .child(
                                                if advanced {
                                                    div()
                                                        .text_sm()
                                                        .child("✓")
                                                } else {
                                                    div()
                                                }
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        // About section with Logo matching localsend (no Card wrapper, direct content)
                        v_flex()
                            .gap(px(5.)) // SizedBox(height: 5) after logo
                            .items_center()
                            .child(crate::ui::components::logo::Logo::new().size(80.).with_text(true))
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
                                    .child("© 2025 NearSend"),
                            )
                            .child(
                                Button::new("changelog")
                                    .ghost()
                                    .on_click(cx.listener(|_this, _event, _window, _cx| {
                                        log::info!("Changelog clicked");
                                    }))
                                    .child("📜 Changelog"),
                            ),
                    )
                    .child(div().h(px(80.))), // SizedBox(height: 80) matching localsend
            )
            .into_any_element()
    }
}
