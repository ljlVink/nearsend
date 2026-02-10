//! History page: full-screen route showing transfer history.

use crate::state::history_state::HistoryState;
use crate::ui::components::history_item::HistoryItem;
use crate::ui::theme::spacing;
use gpui::{div, prelude::*, px, Context, Entity, Window};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex, v_flex, ActiveTheme as _, Icon, Sizable as _, Size, StyledExt as _,
};
use gpui_router::RouterState;

/// History page: back bar + title + history list.
pub struct HistoryPage {
    history_state: Option<Entity<HistoryState>>,
}

impl HistoryPage {
    pub fn new() -> Self {
        Self {
            history_state: None,
        }
    }

    pub fn with_history_state(mut self, state: Entity<HistoryState>) -> Self {
        self.history_state = Some(state);
        self
    }
}

impl gpui::Render for HistoryPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let entries = if let Some(ref state) = self.history_state {
            state.read(cx).entries().to_vec()
        } else {
            vec![]
        };
        let has_entries = !entries.is_empty();

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            // App bar
            .child(
                h_flex()
                    .w_full()
                    .h(px(56.))
                    .px(px(15.))
                    .items_center()
                    .child(
                        Button::new("history-back")
                            .ghost()
                            .child(
                                Icon::default()
                                    .path("icons/arrow-left.svg")
                                    .with_size(Size::Small),
                            )
                            .on_click(cx.listener(|_this, _event, window, cx| {
                                RouterState::global_mut(cx).location.pathname = "/".into();
                                window.refresh();
                            })),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_center()
                            .text_base()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("历史记录"),
                    )
                    .when(has_entries, |this| {
                        this.child(
                            Button::new("history-clear")
                                .ghost()
                                .on_click(cx.listener(|this, _event, _window, cx| {
                                    if let Some(ref state) = this.history_state {
                                        state.update(cx, |s, _cx| s.clear());
                                    }
                                }))
                                .child(div().text_sm().text_color(cx.theme().danger).child("清空")),
                        )
                    })
                    .when(!has_entries, |this| this.child(div().w(px(40.)))),
            )
            // Content
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .overflow_y_scrollbar()
                    .child(if has_entries {
                        v_flex()
                            .w_full()
                            .px(px(15.))
                            .gap(spacing::SM)
                            .children(entries.into_iter().map(|entry| {
                                HistoryItem::new(entry)
                                    .on_open(|_id, _window, _cx| {
                                        log::info!("Open history entry");
                                    })
                                    .on_delete(|id, _window, _cx| {
                                        log::info!("Delete history entry: {}", id);
                                    })
                            }))
                            .into_any_element()
                    } else {
                        div()
                            .flex_1()
                            .w_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .py(px(80.))
                            .child(
                                v_flex()
                                    .items_center()
                                    .gap(spacing::MD)
                                    .child(
                                        Icon::default()
                                            .path("icons/history.svg")
                                            .with_size(Size::Large)
                                            .text_color(cx.theme().muted_foreground),
                                    )
                                    .child(
                                        div()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("暂无记录"),
                                    ),
                            )
                            .into_any_element()
                    }),
            )
    }
}

impl Default for HistoryPage {
    fn default() -> Self {
        Self::new()
    }
}
