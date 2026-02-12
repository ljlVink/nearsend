//! Selected files page: review and manage files before sending.
//! Route: /send/files

use crate::state::app_state::AppState;
use crate::state::send_selection_state::SendSelectionState;
use crate::ui::theme::spacing;
use gpui::{div, prelude::*, px, Context, Entity, Window};
use gpui_component::input::{Input, InputState};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::{
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    h_flex, v_flex, ActiveTheme as _, Icon, Sizable as _, Size, StyledExt as _, WindowExt as _,
};
use gpui_router::RouterState;

enum ClipboardPickOutcome {
    Success(String),
    PermissionDenied,
    ReadFailed,
}

/// Selected files page state.
pub struct SelectedFilesPage {
    app_state: Entity<AppState>,
    send_selection_state: Entity<SendSelectionState>,
}

impl SelectedFilesPage {
    pub fn new(
        app_state: Entity<AppState>,
        send_selection_state: Entity<SendSelectionState>,
    ) -> Self {
        Self {
            app_state,
            send_selection_state,
        }
    }

    fn open_notice_dialog(&self, msg: &str, window: &mut Window, cx: &mut Context<Self>) {
        let msg = msg.to_string();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            dialog
                .title("提示")
                .overlay(true)
                .w(px(320.))
                .child(div().w_full().text_sm().child(msg.clone()))
                .alert()
                .button_props(gpui_component::dialog::DialogButtonProps::default().ok_text("确定"))
        });
    }

    fn open_text_edit_dialog(
        &self,
        index: usize,
        initial: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .auto_grow(3, 5)
                .placeholder("输入文本内容")
                .default_value(initial)
                .soft_wrap(true)
        });
        let send_state = self.send_selection_state.clone();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let input_for_ok = input_state.clone();
            let send_state_for_ok = send_state.clone();
            dialog
                .title("编辑文本")
                .overlay(true)
                .w(px(360.))
                .child(
                    div()
                        .w_full()
                        .child(Input::new(&input_state).appearance(true)),
                )
                .confirm()
                .button_props(
                    gpui_component::dialog::DialogButtonProps::default()
                        .ok_text("确认")
                        .cancel_text("取消"),
                )
                .on_ok(move |_event, _window, cx| {
                    let text = input_for_ok.read(cx).value().to_string();
                    if text.is_empty() {
                        return false;
                    }
                    send_state_for_ok.update(cx, |state, _| {
                        if index == usize::MAX {
                            state.add_text(text.clone());
                        } else {
                            state.update_text(index, text.clone());
                        }
                    });
                    true
                })
        });
    }

    fn open_add_dialog(&self, window: &mut Window, cx: &mut Context<Self>) {
        let page = cx.entity();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let page_text = page.clone();
            let page_file = page.clone();
            let page_folder = page.clone();
            let page_clipboard = page.clone();
            let variant = ButtonCustomVariant::new(_cx)
                .color(_cx.theme().secondary)
                .foreground(_cx.theme().foreground)
                .hover(_cx.theme().secondary)
                .active(_cx.theme().secondary);
            dialog
                .title("你想加入什么文件？")
                .overlay(true)
                .w(px(340.))
                .child(
                    h_flex()
                        .w_full()
                        .gap(px(10.))
                        .flex_wrap()
                        .justify_start()
                        .child(
                            Button::new("selected-add-file")
                                .custom(variant.clone())
                                .w(px(90.))
                                .h(px(65.))
                                .rounded_md()
                                .on_click(move |_event, window, cx| {
                                    window.close_dialog(cx);
                                    page_file.update(cx, |this, cx| {
                                        this.open_notice_dialog("文件选择器即将接入。", window, cx);
                                    });
                                })
                                .child(
                                    v_flex()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(4.))
                                        .child(
                                            Icon::default()
                                                .path("icons/file.svg")
                                                .with_size(gpui_component::Size::Medium)
                                                .text_color(_cx.theme().foreground),
                                        )
                                        .child(div().text_sm().text_center().child("文件")),
                                ),
                        )
                        .child(
                            Button::new("selected-add-folder")
                                .custom(variant.clone())
                                .w(px(90.))
                                .h(px(65.))
                                .rounded_md()
                                .on_click(move |_event, window, cx| {
                                    window.close_dialog(cx);
                                    page_folder.update(cx, |this, cx| {
                                        this.open_notice_dialog("文件夹选择即将接入。", window, cx);
                                    });
                                })
                                .child(
                                    v_flex()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(4.))
                                        .child(
                                            Icon::default()
                                                .path("icons/folder.svg")
                                                .with_size(gpui_component::Size::Medium)
                                                .text_color(_cx.theme().foreground),
                                        )
                                        .child(div().text_sm().text_center().child("文件夹")),
                                ),
                        )
                        .child(
                            Button::new("selected-add-text")
                                .custom(variant.clone())
                                .w(px(90.))
                                .h(px(65.))
                                .rounded_md()
                                .on_click(move |_event, window, cx| {
                                    window.close_dialog(cx);
                                    page_text.update(cx, |this, cx| {
                                        this.open_text_edit_dialog(
                                            usize::MAX,
                                            String::new(),
                                            window,
                                            cx,
                                        );
                                    });
                                })
                                .child(
                                    v_flex()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(4.))
                                        .child(
                                            Icon::default()
                                                .path("icons/book-open.svg")
                                                .with_size(gpui_component::Size::Medium)
                                                .text_color(_cx.theme().foreground),
                                        )
                                        .child(div().text_sm().text_center().child("文本")),
                                ),
                        )
                        .child(
                            Button::new("selected-add-clipboard")
                                .custom(variant)
                                .w(px(90.))
                                .h(px(65.))
                                .rounded_md()
                                .on_click(move |_event, window, cx| {
                                    window.close_dialog(cx);
                                    page_clipboard.update(cx, |this, cx| {
                                        this.add_from_clipboard(window, cx);
                                    });
                                })
                                .child(
                                    v_flex()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(4.))
                                        .child(
                                            Icon::default()
                                                .path("icons/external-link.svg")
                                                .with_size(gpui_component::Size::Medium)
                                                .text_color(_cx.theme().foreground),
                                        )
                                        .child(div().text_sm().text_center().child("剪贴板")),
                                ),
                        ),
                )
                .alert()
                .button_props(gpui_component::dialog::DialogButtonProps::default().ok_text("关闭"))
        });
    }

    fn add_from_clipboard(&self, window: &mut Window, cx: &mut Context<Self>) {
        let window_handle = window.window_handle();
        let page_entity = cx.entity();
        let tokio_handle = self.app_state.read(cx).tokio_handle.clone();

        let join = tokio_handle.spawn(async move {
            let permission_granted =
                match crate::platform::clipboard::ensure_read_clipboard_permission().await {
                    Ok(granted) => granted,
                    Err(err) => {
                        log::error!("ensure read clipboard permission failed: {}", err);
                        false
                    }
                };
            if !permission_granted {
                return ClipboardPickOutcome::PermissionDenied;
            }

            let text = match crate::platform::clipboard::read_clipboard_text().await {
                Ok(text) => text,
                Err(err) => {
                    log::error!("read clipboard text failed: {}", err);
                    return ClipboardPickOutcome::ReadFailed;
                }
            };
            if text.is_empty() {
                return ClipboardPickOutcome::Success(String::new());
            }

            ClipboardPickOutcome::Success(text)
        });

        cx.spawn(async move |_this, cx| {
            let outcome = match join.await {
                Ok(outcome) => outcome,
                Err(err) => {
                    log::error!("clipboard task failed: {}", err);
                    ClipboardPickOutcome::ReadFailed
                }
            };

            match outcome {
                ClipboardPickOutcome::Success(text) => {
                    if text.is_empty() {
                        return;
                    }
                    let _ = page_entity.update(cx, |this, cx| {
                        this.send_selection_state.update(cx, |state, _| {
                            state.add_text(text.clone());
                        });
                    });
                }
                ClipboardPickOutcome::PermissionDenied => {
                    let _ = window_handle.update(cx, |_, window, cx| {
                        let _ = page_entity.update(cx, |this, cx| {
                            this.open_notice_dialog("无权限。请开启权限。", window, cx);
                        });
                    });
                }
                ClipboardPickOutcome::ReadFailed => {
                    let _ = window_handle.update(cx, |_, window, cx| {
                        let _ = page_entity.update(cx, |this, cx| {
                            this.open_notice_dialog("读取剪贴板失败。", window, cx);
                        });
                    });
                }
            }
        })
        .detach();
    }
}

impl gpui::Render for SelectedFilesPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let files = self.send_selection_state.read(cx).items().to_vec();
        let total_size = self.send_selection_state.read(cx).total_size();
        let file_count = files.len();
        let send_state = self.send_selection_state.clone();

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                h_flex()
                    .w_full()
                    .h(px(56.))
                    .px(px(15.))
                    .items_center()
                    .child(
                        Button::new("files-back")
                            .ghost()
                            .on_click(cx.listener(|_this, _event, window, cx| {
                                RouterState::global_mut(cx).location.pathname = "/".into();
                                window.refresh();
                            }))
                            .child(
                                Icon::default()
                                    .path("icons/arrow-left.svg")
                                    .with_size(Size::Small),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_center()
                            .text_base()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("选择"),
                    )
                    .child(div().w(px(40.))),
            )
            .child(
                div().flex_1().w_full().overflow_y_scrollbar().child(
                    v_flex()
                        .w_full()
                        .px(px(15.))
                        .gap(spacing::SM)
                        .child(
                            h_flex()
                                .w_full()
                                .justify_between()
                                .items_center()
                                .child(
                                    v_flex()
                                        .gap(px(2.))
                                        .child(
                                            div()
                                                .text_lg()
                                                .text_color(cx.theme().foreground)
                                                .child(format!("文件： {}", file_count)),
                                        )
                                        .child(
                                            div()
                                                .text_lg()
                                                .text_color(cx.theme().foreground)
                                                .child(format!(
                                                    "大小： {}",
                                                    format_file_size(total_size)
                                                )),
                                        ),
                                )
                                .child(
                                    Button::new("files-clear-all")
                                        .primary()
                                        .on_click(cx.listener(
                                            move |_this, _event, _window, _cx| {
                                                send_state.update(_cx, |state, _| {
                                                    state.clear();
                                                });
                                            },
                                        ))
                                        .child("全部删除"),
                                ),
                        )
                        .children(files.iter().enumerate().map(|(i, file)| {
                            let file_name = file.name.clone();
                            let file_size = format_file_size(file.size);
                            let is_text = file.text_content.is_some();
                            let text = file.text_content.clone().unwrap_or_default();
                            let send_state_for_delete = self.send_selection_state.clone();
                            div()
                                .bg(cx.theme().secondary)
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_lg()
                                .p(px(12.))
                                .child(
                                    h_flex()
                                        .items_center()
                                        .gap(spacing::SM)
                                        .w_full()
                                        .child(
                                            div()
                                                .w(px(56.))
                                                .h(px(56.))
                                                .rounded_md()
                                                .bg(cx.theme().primary.opacity(0.18))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(
                                                    Icon::default()
                                                        .path("icons/book-open.svg")
                                                        .with_size(Size::Medium)
                                                        .text_color(cx.theme().foreground),
                                                ),
                                        )
                                        .child(
                                            v_flex()
                                                .flex_1()
                                                .gap(px(2.))
                                                .child(
                                                    div()
                                                        .text_base()
                                                        .text_color(cx.theme().foreground)
                                                        .child(format!("\"{}\"", file_name)),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(file_size),
                                                ),
                                        )
                                        .child(
                                            h_flex()
                                                .gap(px(8.))
                                                .when(is_text, |this| {
                                                    this.child(
                                                        Button::new(format!("edit-file-{}", i))
                                                            .ghost()
                                                            .on_click(cx.listener(
                                                                move |this, _event, window, cx| {
                                                                    this.open_text_edit_dialog(
                                                                        i,
                                                                        text.clone(),
                                                                        window,
                                                                        cx,
                                                                    );
                                                                },
                                                            ))
                                                            .child("编辑"),
                                                    )
                                                })
                                                .child(
                                                    Button::new(format!("delete-file-{}", i))
                                                        .ghost()
                                                        .on_click(cx.listener(
                                                            move |_this, _event, _window, _cx| {
                                                                send_state_for_delete.update(
                                                                    _cx,
                                                                    |state, _| {
                                                                        state.remove(i);
                                                                    },
                                                                );
                                                            },
                                                        ))
                                                        .child(
                                                            Icon::default()
                                                                .path("icons/trash.svg")
                                                                .with_size(Size::Small)
                                                                .text_color(cx.theme().danger),
                                                        ),
                                                ),
                                        ),
                                )
                        }))
                        .when(files.is_empty(), |this| {
                            this.child(
                                div()
                                    .w_full()
                                    .py(px(40.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("暂无文件"),
                            )
                        }),
                ),
            )
            .child(
                div().w_full().px(px(15.)).py(px(15.)).child(
                    h_flex().justify_end().items_center().child(
                        Button::new("add-more-files")
                            .primary()
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.open_add_dialog(window, cx);
                            }))
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .child(
                                        Icon::default()
                                            .path("icons/plus.svg")
                                            .with_size(Size::Small),
                                    )
                                    .child("添加"),
                            ),
                    ),
                ),
            )
    }
}

fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
