//! Settings interactions and address-target dialogs.

use super::*;

impl HomePage {
    pub(super) fn init_select_states(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Theme select: 系统 / 浅色 / 深色
        let theme_idx = match self.settings_state.theme_mode {
            ThemeMode::System => 0,
            ThemeMode::Light => 1,
            ThemeMode::Dark => 2,
        };
        let theme_select = cx.new(|cx| {
            SelectState::new(
                vec!["系统", "浅色", "深色"],
                Some(IndexPath::default().row(theme_idx)),
                window,
                cx,
            )
        });
        cx.subscribe_in(
            &theme_select,
            window,
            |this, _, event: &SelectEvent<Vec<&'static str>>, _win, _cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.settings_state.theme_mode = match *value {
                        "浅色" => ThemeMode::Light,
                        "深色" => ThemeMode::Dark,
                        _ => ThemeMode::System,
                    };
                    this.persist_settings();
                }
            },
        )
        .detach();
        self.theme_select = Some(theme_select);

        // Color select: 系统 / NearSend / OLED
        let color_idx = match self.settings_state.color_mode {
            ColorMode::System => 0,
            ColorMode::LocalSend => 1,
            ColorMode::Oled => 2,
        };
        let color_select = cx.new(|cx| {
            SelectState::new(
                vec!["系统", "NearSend", "OLED"],
                Some(IndexPath::default().row(color_idx)),
                window,
                cx,
            )
        });
        cx.subscribe_in(
            &color_select,
            window,
            |this, _, event: &SelectEvent<Vec<&'static str>>, _win, _cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.settings_state.color_mode = match *value {
                        "NearSend" => ColorMode::LocalSend,
                        "OLED" => ColorMode::Oled,
                        _ => ColorMode::System,
                    };
                    this.persist_settings();
                }
            },
        )
        .detach();
        self.color_select = Some(color_select);

        // Language select
        let language_select = cx.new(|cx| {
            SelectState::new(
                vec!["系统", "简体中文", "English", "日本語"],
                Some(IndexPath::default()),
                window,
                cx,
            )
        });
        cx.subscribe_in(
            &language_select,
            window,
            |this, _, event: &SelectEvent<Vec<&'static str>>, _win, _cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.settings_state.language = value.to_string();
                    this.persist_settings();
                }
            },
        )
        .detach();
        self.language_select = Some(language_select);
    }

    /// Opens a dialog with a multiline text input for sending text messages.
    /// Matches LocalSend's MessageInputDialog behavior.
    pub(super) fn open_text_input_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .auto_grow(3, 5)
                .placeholder("输入文本内容")
                .soft_wrap(true)
        });
        self.text_input_state = Some(input_state.clone());

        let home_entity = cx.entity();

        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();

            dialog
                .title("发送文本")
                .overlay(true)
                .w(px(340.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true)),
                )
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("确认")
                        .show_cancel(true)
                        .cancel_text("取消"),
                )
                .footer(Self::build_confirm_dialog_footer(
                    "text-input",
                    "确认",
                    "取消",
                ))
                .on_ok(move |_event, _window, cx| {
                    let text = input_for_ok.read(cx).value().to_string();
                    if !text.is_empty() {
                        home_for_ok.update(cx, |this, _cx| {
                            this.send_selection_state.update(_cx, |state, _| {
                                state.add_text(text.clone());
                            });
                        });
                    }
                    true
                })
        });
    }

    pub(super) fn open_send_pin_dialog(
        &mut self,
        show_invalid_pin: bool,
        responder: oneshot::Sender<Option<String>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("输入接收端 PIN"));
        let responder = Arc::new(Mutex::new(Some(responder)));
        let home_entity = cx.entity();

        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let responder_for_ok = responder.clone();
            let responder_for_cancel = responder.clone();
            let home_for_ok = home_entity.clone();
            let variant = ButtonCustomVariant::new(_cx)
                .color(_cx.theme().secondary)
                .foreground(_cx.theme().foreground)
                .hover(_cx.theme().secondary)
                .active(_cx.theme().secondary);
            dialog
                .title(if show_invalid_pin {
                    "PIN 错误，请重试"
                } else {
                    "请输入接收端 PIN"
                })
                .overlay(true)
                .w(px(320.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .child(
                    h_flex()
                        .w_full()
                        .justify_end()
                        .gap(px(8.))
                        .child(
                            Button::new("send-pin-cancel")
                                .custom(variant.clone())
                                .on_click(move |_event, window, cx| {
                                    if let Ok(mut guard) = responder_for_cancel.lock() {
                                        if let Some(tx) = guard.take() {
                                            let _ = tx.send(None);
                                        }
                                    }
                                    window.close_dialog(cx);
                                })
                                .child("取消"),
                        )
                        .child(
                            Button::new("send-pin-confirm")
                                .custom(variant)
                                .on_click(move |_event, window, cx| {
                                    let pin = input_for_ok.read(cx).value().trim().to_string();
                                    if pin.is_empty() {
                                        home_for_ok.update(cx, |this, cx| {
                                            this.open_simple_notice_dialog(
                                                "PIN 不能为空",
                                                window,
                                                cx,
                                            );
                                        });
                                        return;
                                    }
                                    if let Ok(mut guard) = responder_for_ok.lock() {
                                        if let Some(tx) = guard.take() {
                                            let _ = tx.send(Some(pin));
                                        }
                                    }
                                    window.close_dialog(cx);
                                })
                                .child("确认"),
                        ),
                )
        });
    }

    pub(super) fn open_server_alias_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("输入设备别名"));
        let current_alias = self.settings_state.server_alias.clone();
        input_state.update(cx, |state, cx| {
            state.set_value(current_alias, window, cx);
        });
        let home_entity = cx.entity();

        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();

            dialog
                .title("编辑别名")
                .overlay(true)
                .w(px(340.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("保存")
                        .show_cancel(true)
                        .cancel_text("取消"),
                )
                .footer(Self::build_confirm_dialog_footer(
                    "server-alias",
                    "保存",
                    "取消",
                ))
                .on_ok(move |_event, window, cx| {
                    let alias = input_for_ok.read(cx).value().trim().to_string();
                    if alias.is_empty() {
                        home_for_ok.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("别名不能为空", window, cx);
                        });
                        return false;
                    }
                    home_for_ok.update(cx, |this, cx| {
                        this.settings_state.server_alias = alias.clone();
                        this.sync_server_config_to_runtime(cx);
                        this.persist_settings();
                    });
                    true
                })
        });
    }

    pub(super) fn open_server_port_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("输入端口号"));
        let current_port = self.settings_state.server_port.to_string();
        input_state.update(cx, |state, cx| {
            state.set_value(current_port, window, cx);
        });
        let home_entity = cx.entity();

        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();

            dialog
                .title("编辑端口")
                .overlay(true)
                .w(px(340.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("保存")
                        .show_cancel(true)
                        .cancel_text("取消"),
                )
                .footer(Self::build_confirm_dialog_footer(
                    "server-port",
                    "保存",
                    "取消",
                ))
                .on_ok(move |_event, window, cx| {
                    let raw = input_for_ok.read(cx).value().trim().to_string();
                    let Ok(port) = raw.parse::<u16>() else {
                        home_for_ok.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("端口必须是 1-65535 的数字", window, cx);
                        });
                        return false;
                    };
                    if port == 0 {
                        home_for_ok.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("端口必须是 1-65535 的数字", window, cx);
                        });
                        return false;
                    }
                    home_for_ok.update(cx, |this, cx| {
                        this.settings_state.server_port = port;
                        this.sync_server_config_to_runtime(cx);
                        this.persist_settings();
                    });
                    true
                })
        });
    }

    pub(super) fn open_receive_pin_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("输入接收 PIN"));
        let current_pin = self.settings_state.receive_pin.clone();
        input_state.update(cx, |state, cx| {
            state.set_value(current_pin, window, cx);
        });
        let home_entity = cx.entity();

        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();

            dialog
                .title("设置接收 PIN")
                .overlay(true)
                .w(px(340.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("保存")
                        .show_cancel(true)
                        .cancel_text("取消"),
                )
                .footer(Self::build_confirm_dialog_footer(
                    "receive-pin",
                    "保存",
                    "取消",
                ))
                .on_ok(move |_event, window, cx| {
                    let pin = input_for_ok.read(cx).value().trim().to_string();
                    if pin.is_empty() {
                        home_for_ok.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("PIN 不能为空", window, cx);
                        });
                        return false;
                    }
                    home_for_ok.update(cx, |this, cx| {
                        this.settings_state.receive_pin = pin.clone();
                        this.sync_server_config_to_runtime(cx);
                        this.persist_settings();
                    });
                    true
                })
        });
    }

    pub(super) fn pick_receive_destination(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let window_handle = window.window_handle();
        let home_entity = cx.entity();
        let tokio_handle = self.app_state.read(cx).tokio_handle.clone();
        let join = tokio_handle
            .spawn(async move { crate::platform::file_picker::pick_save_directory().await });
        cx.spawn(async move |_this, cx| {
            let picked = match join.await {
                Ok(Ok(path)) => path,
                Ok(Err(err)) => {
                    log::error!("pick receive destination failed: {}", err);
                    None
                }
                Err(err) => {
                    log::error!("pick receive destination task failed: {}", err);
                    None
                }
            };
            if let Some(path) = picked {
                let path_text = path.to_string_lossy().to_string();
                let _ = home_entity.update(cx, |this, cx| {
                    this.settings_state.destination = Some(path_text);
                    this.sync_server_config_to_runtime(cx);
                    this.persist_settings();
                });
            } else {
                let _ = window_handle.update(cx, |_, window, cx| {
                    let _ = home_entity.update(cx, |this, cx| {
                        this.open_simple_notice_dialog(
                            "未选择接收目录，保持当前配置。",
                            window,
                            cx,
                        );
                    });
                });
            }
        })
        .detach();
    }

    pub(super) fn clear_receive_destination(&mut self, cx: &mut Context<Self>) {
        self.settings_state.destination = None;
        self.sync_server_config_to_runtime(cx);
        self.persist_settings();
    }

    pub(super) fn cycle_device_type_setting(&mut self, cx: &mut Context<Self>) {
        let next = match normalize_device_type_label(&self.settings_state.device_type).as_str() {
            "mobile" => "Desktop",
            "desktop" => "Web",
            "web" => "Server",
            "server" => "Headless",
            _ => "Mobile",
        };
        self.settings_state.device_type = next.to_string();
        self.sync_server_config_to_runtime(cx);
        self.persist_settings();
    }

    pub(super) fn open_device_model_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入设备型号（可选）"));
        let current = self.settings_state.device_model.clone();
        input_state.update(cx, |state, cx| {
            state.set_value(current, window, cx);
        });
        let home_entity = cx.entity();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();
            dialog
                .title("设备型号")
                .overlay(true)
                .w(px(340.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("保存")
                        .show_cancel(true)
                        .cancel_text("取消"),
                )
                .footer(Self::build_confirm_dialog_footer(
                    "device-model",
                    "保存",
                    "取消",
                ))
                .on_ok(move |_event, _window, cx| {
                    let value = input_for_ok.read(cx).value().trim().to_string();
                    home_for_ok.update(cx, |this, cx| {
                        this.settings_state.device_model = value;
                        this.sync_server_config_to_runtime(cx);
                        this.persist_settings();
                    });
                    true
                })
        });
    }

    pub(super) fn open_discovery_timeout_dialog(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入发现超时（毫秒）"));
        let current = self.settings_state.discovery_timeout.to_string();
        input_state.update(cx, |state, cx| {
            state.set_value(current, window, cx);
        });
        let home_entity = cx.entity();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();
            dialog
                .title("发现超时")
                .overlay(true)
                .w(px(340.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("保存")
                        .show_cancel(true)
                        .cancel_text("取消"),
                )
                .footer(Self::build_confirm_dialog_footer(
                    "discovery-timeout",
                    "保存",
                    "取消",
                ))
                .on_ok(move |_event, window, cx| {
                    let raw = input_for_ok.read(cx).value().trim().to_string();
                    let Ok(value) = raw.parse::<u32>() else {
                        home_for_ok.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("请输入有效数字", window, cx);
                        });
                        return false;
                    };
                    home_for_ok.update(cx, |this, cx| {
                        this.settings_state.discovery_timeout = value.max(200);
                        this.persist_settings();
                    });
                    true
                })
        });
    }

    pub(super) fn open_multicast_group_dialog(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("输入组播地址"));
        let current = self.settings_state.multicast_group.clone();
        input_state.update(cx, |state, cx| {
            state.set_value(current, window, cx);
        });
        let home_entity = cx.entity();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();
            dialog
                .title("组播地址")
                .overlay(true)
                .w(px(340.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("保存")
                        .show_cancel(true)
                        .cancel_text("取消"),
                )
                .footer(Self::build_confirm_dialog_footer(
                    "multicast-group",
                    "保存",
                    "取消",
                ))
                .on_ok(move |_event, window, cx| {
                    let raw = input_for_ok.read(cx).value().trim().to_string();
                    if raw.is_empty() {
                        home_for_ok.update(cx, |this, cx| {
                            this.open_simple_notice_dialog("组播地址不能为空", window, cx);
                        });
                        return false;
                    }
                    home_for_ok.update(cx, |this, cx| {
                        this.settings_state.multicast_group = raw;
                        this.persist_settings();
                    });
                    true
                })
        });
    }

    pub(super) fn cycle_network_filter_mode(&mut self, cx: &mut Context<Self>) {
        self.settings_state.network_filter_mode = match self.settings_state.network_filter_mode {
            NetworkFilterMode::All => NetworkFilterMode::Whitelist,
            NetworkFilterMode::Whitelist => NetworkFilterMode::Blacklist,
            NetworkFilterMode::Blacklist => NetworkFilterMode::All,
        };
        self.sync_server_config_to_runtime(cx);
        self.persist_settings();
    }

    pub(super) fn open_network_filters_dialog(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("每行一个规则：192.168.1.*"));
        let current = self.settings_state.network_filters.join("\n");
        input_state.update(cx, |state, cx| {
            state.set_value(current, window, cx);
        });
        let home_entity = cx.entity();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let home_for_ok = home_entity.clone();
            dialog
                .title("网络接口过滤规则")
                .overlay(true)
                .w(px(380.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true).large()),
                )
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("保存")
                        .show_cancel(true)
                        .cancel_text("取消"),
                )
                .footer(Self::build_confirm_dialog_footer(
                    "network-filters",
                    "保存",
                    "取消",
                ))
                .on_ok(move |_event, _window, cx| {
                    let raw = input_for_ok.read(cx).value().to_string();
                    let filters = raw
                        .lines()
                        .map(|line| line.trim().to_string())
                        .filter(|line| !line.is_empty())
                        .collect::<Vec<_>>();
                    home_for_ok.update(cx, |this, cx| {
                        this.settings_state.network_filters = filters;
                        this.sync_server_config_to_runtime(cx);
                        this.persist_settings();
                    });
                    true
                })
        });
    }

    /// Opens LocalSend-like address input dialog (Label/IP tabs, single input).
    pub(super) fn open_send_to_address_dialog(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_send_to_address_dialog_with_mode(AddressInputMode::Label, window, cx);
    }

    fn open_send_to_address_dialog_with_mode(
        &mut self,
        mode: AddressInputMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let placeholder = match mode {
            AddressInputMode::Label => "#123",
            AddressInputMode::IpAddress => "输入IP地址",
        };
        let ip_input_state = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder));
        self.send_ip_input_state = Some(ip_input_state.clone());
        let home_entity = cx.entity();
        let prefixes = self.local_ip_prefixes();
        let example_text = match mode {
            AddressInputMode::Label => {
                if prefixes.is_empty() {
                    "示例：123\n可用网段：\n- 192.168.1.".to_string()
                } else {
                    let mut text = "示例：123\n可用网段：".to_string();
                    for p in prefixes.iter().take(3) {
                        text.push_str(&format!("\n- {}.", p));
                    }
                    text
                }
            }
            AddressInputMode::IpAddress => {
                if prefixes.is_empty() {
                    "示例：\n- 192.168.1.23\n- 192.168.1.123".to_string()
                } else {
                    let mut text = "示例：".to_string();
                    for p in prefixes.iter().take(3) {
                        text.push_str(&format!("\n- {}.123", p));
                    }
                    text
                }
            }
        };
        let tag_tab_style = ButtonCustomVariant::new(cx)
            .color(if mode == AddressInputMode::Label {
                cx.theme().primary.opacity(0.2)
            } else {
                cx.theme().secondary
            })
            .foreground(if mode == AddressInputMode::Label {
                cx.theme().primary
            } else {
                cx.theme().foreground
            })
            .hover(if mode == AddressInputMode::Label {
                cx.theme().primary.opacity(0.2)
            } else {
                cx.theme().secondary
            })
            .active(if mode == AddressInputMode::Label {
                cx.theme().primary.opacity(0.2)
            } else {
                cx.theme().secondary
            });
        let ip_tab_style = ButtonCustomVariant::new(cx)
            .color(if mode == AddressInputMode::IpAddress {
                cx.theme().primary.opacity(0.2)
            } else {
                cx.theme().secondary
            })
            .foreground(if mode == AddressInputMode::IpAddress {
                cx.theme().primary
            } else {
                cx.theme().foreground
            })
            .hover(if mode == AddressInputMode::IpAddress {
                cx.theme().primary.opacity(0.2)
            } else {
                cx.theme().secondary
            })
            .active(if mode == AddressInputMode::IpAddress {
                cx.theme().primary.opacity(0.2)
            } else {
                cx.theme().secondary
            });

        window.open_dialog(cx, move |dialog, _window, _cx| {
            let ip_for_ok = ip_input_state.clone();
            let home_for_ok = home_entity.clone();
            let home_for_tag_tab = home_entity.clone();
            let home_for_ip_tab = home_entity.clone();
            let mode_for_ok = mode;

            dialog
                .title("输入地址")
                .overlay(true)
                .w(px(340.))
                .child(
                    v_flex()
                        .w_full()
                        .gap(px(12.))
                        .child(
                            h_flex()
                                .gap(px(0.))
                                .child(
                                    Button::new("address-mode-label")
                                        .custom(tag_tab_style.clone())
                                        .w(px(72.))
                                        .h(px(32.))
                                        .rounded_l(px(12.))
                                        .rounded_r(px(0.))
                                        .on_click(move |_event, window, cx| {
                                            if mode != AddressInputMode::Label {
                                                window.close_dialog(cx);
                                                home_for_tag_tab.update(cx, |this, cx| {
                                                    this.open_send_to_address_dialog_with_mode(
                                                        AddressInputMode::Label,
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            }
                                        })
                                        .child(div().text_sm().font_medium().child("标签")),
                                )
                                .child(
                                    Button::new("address-mode-ip")
                                        .custom(ip_tab_style.clone())
                                        .w(px(88.))
                                        .h(px(32.))
                                        .rounded_l(px(0.))
                                        .rounded_r(px(12.))
                                        .on_click(move |_event, window, cx| {
                                            if mode != AddressInputMode::IpAddress {
                                                window.close_dialog(cx);
                                                home_for_ip_tab.update(cx, |this, cx| {
                                                    this.open_send_to_address_dialog_with_mode(
                                                        AddressInputMode::IpAddress,
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            }
                                        })
                                        .child(div().text_sm().font_medium().child("IP 地址")),
                                ),
                        )
                        .child(
                            div()
                                .w_full()
                                .shadow_xs()
                                .rounded_md()
                                .child(Input::new(&ip_input_state).appearance(true).large()),
                        )
                        .child(
                            div()
                                .w_full()
                                .text_sm()
                                .text_color(_cx.theme().muted_foreground)
                                .child(example_text.clone()),
                        ),
                )
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("确认")
                        .show_cancel(true)
                        .cancel_text("取消"),
                )
                .footer(Self::build_confirm_dialog_footer(
                    "send-to-address",
                    "确认",
                    "取消",
                ))
                .on_ok(move |_event, window, cx| {
                    let raw = ip_for_ok.read(cx).value().trim().to_string();
                    if raw.is_empty() {
                        return false;
                    }
                    home_for_ok.update(cx, |this, cx| {
                        let port = this.settings_state.server_port;
                        match mode_for_ok {
                            AddressInputMode::IpAddress => {
                                this.execute_send(raw.clone(), port, window, cx);
                            }
                            AddressInputMode::Label => {
                                if let Some(ip) = this.resolve_labeled_ip(&raw) {
                                    this.execute_send(ip, port, window, cx);
                                } else {
                                    this.open_simple_notice_dialog(
                                        "无法根据标签推导可用 IP，请切换到“IP 地址”模式直接输入。",
                                        window,
                                        cx,
                                    );
                                }
                            }
                        }
                    });
                    true
                })
        });
    }

    fn resolve_labeled_ip(&self, label: &str) -> Option<String> {
        let suffix = label.trim().trim_start_matches('#');
        let suffix_num: u8 = suffix.parse().ok()?;
        let prefixes = self.local_ip_prefixes();
        let prefix = prefixes.first()?;
        Some(format!("{}.{}", prefix, suffix_num))
    }

    fn local_ip_prefixes(&self) -> Vec<String> {
        let mut prefixes = BTreeSet::new();
        for ip in &self.send_state.local_ips {
            if let Some(p) = ipv4_prefix(ip) {
                prefixes.insert(p);
            }
        }
        if let Some(ip) = detect_primary_route_ipv4() {
            if let Some(p) = ipv4_prefix(&ip.to_string()) {
                prefixes.insert(p);
            }
        }
        prefixes.into_iter().collect()
    }

    pub(super) fn open_send_target_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_send_to_address_dialog(window, cx);
    }

}
