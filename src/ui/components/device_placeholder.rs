use crate::ui::components::device_badge::DeviceBadge;
use crate::ui::theme::{sizing, spacing};
use gpui::{div, prelude::*, px, IntoElement, Window};
use gpui_component::{h_flex, v_flex, ActiveTheme as _, StyledExt as _};

/// Device placeholder component matching localsend's DevicePlaceholderListTile
#[derive(IntoElement)]
pub struct DevicePlaceholder;

impl gpui::RenderOnce for DevicePlaceholder {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        div()
            .bg(cx.theme().secondary)
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p(sizing::CARD_PADDING)
            .mb(spacing::MD)
            .opacity(30.0)
            .child(
                div().h(px(60.)).w_full().child(
                    h_flex()
                        .gap(spacing::MD)
                        .items_center()
                        .child(
                            // Placeholder icon
                            div()
                                .w(px(46.))
                                .h(px(46.))
                                .bg(cx.theme().muted)
                                .rounded_md(),
                        )
                        .child(
                            v_flex().gap(px(10.)).flex_1().child(
                                // Placeholder badges
                                h_flex()
                                    .gap(px(10.))
                                    .child(
                                        DeviceBadge::new("       ")
                                            .background_color(cx.theme().muted.into())
                                            .foreground_color(cx.theme().muted.into()),
                                    )
                                    .child(
                                        DeviceBadge::new("              ")
                                            .background_color(cx.theme().muted.into())
                                            .foreground_color(cx.theme().muted.into()),
                                    ),
                            ),
                        ),
                ),
            )
    }
}
