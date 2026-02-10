use crate::ui::components::device_badge::DeviceBadge;
use crate::ui::theme::{sizing, spacing};
use gpui::{div, prelude::*, px, Animation, AnimationExt as _, IntoElement, Window};
use gpui_component::{h_flex, v_flex, ActiveTheme as _, Icon, Sizable as _, Size};
use std::time::Duration;

/// Device icon paths that cycle through (matching LocalSend's DeviceType rotation).
const DEVICE_ICONS: &[&str] = &[
    "icons/smartphone.svg",
    "icons/monitor.svg",
    "icons/globe.svg",
    "icons/server.svg",
];

/// Device placeholder component matching LocalSend's DevicePlaceholderListTile.
/// Shows a single card with rotating device type icons and skeleton badges.
#[derive(Clone, Copy, IntoElement)]
pub struct DevicePlaceholder;

impl gpui::RenderOnce for DevicePlaceholder {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let muted = cx.theme().muted;
        let icon_count = DEVICE_ICONS.len();
        let cycle_ms: u64 = 3000 * icon_count as u64;

        div()
            .bg(cx.theme().secondary)
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p(sizing::CARD_PADDING)
            .mb(spacing::MD)
            .opacity(0.5)
            .child(
                h_flex()
                    .gap(spacing::MD)
                    .items_center()
                    .child(
                        // Rotating device icon
                        div()
                            .w(px(46.))
                            .h(px(46.))
                            .bg(muted)
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .with_animation(
                                "device-icon-rotate",
                                Animation::new(Duration::from_millis(cycle_ms)).repeat(),
                                move |this, delta| {
                                    let elapsed = delta * cycle_ms as f32;
                                    let per_icon_ms = 3000.0_f32;
                                    let mut idx = (elapsed / per_icon_ms).floor() as usize;
                                    if idx >= icon_count {
                                        idx = icon_count - 1;
                                    }
                                    // Fade in/out
                                    let local = elapsed - (idx as f32 * per_icon_ms);
                                    let fade = 300.0_f32;
                                    let alpha = if local < fade {
                                        local / fade
                                    } else if local > per_icon_ms - fade {
                                        (per_icon_ms - local) / fade
                                    } else {
                                        1.0
                                    };
                                    this.opacity(alpha.clamp(0.0, 1.0)).child(
                                        Icon::default()
                                            .path(DEVICE_ICONS[idx])
                                            .with_size(Size::Large),
                                    )
                                },
                            ),
                    )
                    .child(
                        v_flex()
                            .gap(px(8.))
                            .flex_1()
                            .child(
                                // Name placeholder bar
                                div().w(px(100.)).h(px(14.)).bg(muted).rounded(px(4.)),
                            )
                            .child(
                                h_flex()
                                    .gap(px(10.))
                                    .child(
                                        DeviceBadge::new("       ")
                                            .background_color(muted.into())
                                            .foreground_color(gpui::rgba(0x00000000)),
                                    )
                                    .child(
                                        DeviceBadge::new("              ")
                                            .background_color(muted.into())
                                            .foreground_color(gpui::rgba(0x00000000)),
                                    ),
                            ),
                    ),
            )
    }
}
