//! Changelog page rendered from project `changelog.md`.

use crate::ui::routes;
use gpui::{div, prelude::*, px, Context, Entity, Window};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::{
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    h_flex,
    text::markdown,
    v_flex, ActiveTheme as _, Icon, Sizable as _, Size, StyledExt as _,
};

pub struct ChangelogPage {
    pub root: Option<Entity<crate::app::AppRoot>>,
}

impl ChangelogPage {
    pub fn new(root: Entity<crate::app::AppRoot>) -> Self {
        Self { root: Some(root) }
    }
}

impl gpui::Render for ChangelogPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let back_button_variant = ButtonCustomVariant::new(cx)
            .hover(cx.theme().transparent)
            .active(cx.theme().transparent);

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                h_flex()
                    .w_full()
                    .h(px(56.))
                    .px(px(16.))
                    .items_center()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        h_flex()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                Button::new("changelog-back")
                                    .ghost()
                                    .custom(back_button_variant)
                                    .h(px(36.))
                                    .w(px(36.))
                                    .p(px(0.))
                                    .rounded_md()
                                    .child(
                                        Icon::default()
                                            .path("icons/arrow-left.svg")
                                            .with_size(Size::Small),
                                    )
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        if let Some(root) = &this.root {
                                            let _ = root.update(cx, |root, cx| {
                                                root.go_back_or_navigate(routes::HOME, cx);
                                            });
                                        }
                                    })),
                            )
                            .child(
                                div()
                                    .text_lg()
                                    .font_semibold()
                                    .text_color(cx.theme().foreground)
                                    .child("更新日志"),
                            ),
                    ),
            )
            .child(
                div().flex_1().w_full().overflow_y_scrollbar().child(
                    v_flex()
                        .w_full()
                        .px(px(15.))
                        .py(px(12.))
                        .child(
                            div()
                                .w_full()
                                .rounded_lg()
                                .border_1()
                                .border_color(cx.theme().border)
                                .bg(cx.theme().secondary)
                                .p(px(14.))
                                .child(
                                    markdown(include_str!("../../../changelog.md"))
                                        .selectable(true)
                                        .scrollable(false),
                                ),
                        )
                        .child(div().h(px(24.))),
                ),
            )
    }
}
