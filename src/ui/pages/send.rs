use crate::state::device_state::DeviceState;
use crate::ui::components::{device_card::DeviceCard, file_list::FileList};
use crate::ui::theme::{sizing, spacing};
use gpui::{div, prelude::*, px, Context, Entity, SharedString, Window};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    scroll::ScrollableElement,
    v_flex, ActiveTheme as _, Sizable as _, StyledExt as _,
};
use localsend::http::state::ClientInfo;

/// Send page for selecting files and devices (mobile-first design)
/// Aligned with LocalSend mobile UI
pub struct SendPage {
    device_state: Entity<DeviceState>,
    selected_files: Vec<std::path::PathBuf>,
}

impl SendPage {
    pub fn new(device_state: Entity<DeviceState>) -> Self {
        Self {
            device_state,
            selected_files: Vec::new(),
        }
    }
}

impl gpui::Render for SendPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let device_state = self.device_state.clone();
        let selected_files = self.selected_files.clone();
        let has_files = !selected_files.is_empty();

        div()
            .size_full()
            .w_full()
            .bg(cx.theme().background)
            .overflow_y_scrollbar()
            .child(
                v_flex()
                    .w_full()
                    .p(spacing::MD)
                    .gap(spacing::LG)
                    .child(
                        // File selection section
                        v_flex()
                            .gap(spacing::MD)
                            .w_full()
                            .child(
                                div()
                                    .text_lg()
                                    .font_semibold()
                                    .text_color(cx.theme().foreground)
                                    .child("Select Files"),
                            )
                            .when(!has_files, |this| {
                                // File selection buttons (when no files selected)
                                this.child(
                                    v_flex()
                                        .gap(spacing::MD)
                                        .w_full()
                                        .child(
                                            Button::new("photos")
                                                .with_variant(gpui_component::button::ButtonVariant::Secondary)
                                                .outline()
                                                .w_full()
                                                .h(sizing::BUTTON_HEIGHT)
                                                .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                    // TODO: Implement photo picker
                                                    log::info!("Photos clicked");
                                                }))
                                                .child("📷 Photos"),
                                        )
                                        .child(
                                            Button::new("videos")
                                                .with_variant(gpui_component::button::ButtonVariant::Secondary)
                                                .outline()
                                                .w_full()
                                                .h(sizing::BUTTON_HEIGHT)
                                                .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                    // TODO: Implement video picker
                                                    log::info!("Videos clicked");
                                                }))
                                                .child("🎥 Videos"),
                                        )
                                        .child(
                                            Button::new("files")
                                                .with_variant(gpui_component::button::ButtonVariant::Secondary)
                                                .outline()
                                                .w_full()
                                                .h(sizing::BUTTON_HEIGHT)
                                                .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                    // TODO: Implement file picker
                                                    log::info!("Files clicked");
                                                }))
                                                .child("📄 Files"),
                                        ),
                                )
                            })
                            .when(has_files, |this| {
                                // Selected files card (when files are selected)
                                this.child(
                                    div()
                                        .bg(cx.theme().secondary)
                                        .border_1()
                                        .border_color(cx.theme().border)
                                        .rounded_lg()
                                        .p(spacing::MD)
                                        .child(
                                            v_flex()
                                                .gap(spacing::MD)
                                                .child(
                                                    // Header with clear button
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
                                                                    this.selected_files.clear();
                                                                }))
                                                                .child("✕"),
                                                        ),
                                                )
                                                .child(
                                                    // File count and size
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
                                                                .child("0 B"), // TODO: Calculate total size
                                                        ),
                                                )
                                                .child(
                                                    // File thumbnails (horizontal scroll)
                                                    div()
                                                        .h(px(80.))
                                                        .overflow_x_scrollbar()
                                                        .child(
                                                            h_flex()
                                                                .gap(spacing::SM)
                                                                .children(selected_files.iter().enumerate().map(|(i, _path)| {
                                                                    div()
                                                                        .w(px(80.))
                                                                        .h(px(80.))
                                                                        .bg(cx.theme().muted)
                                                                        .rounded_md()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .child(format!("File {}", i + 1))
                                                                })),
                                                        ),
                                                )
                                                .child(
                                                    // Action buttons
                                                    h_flex()
                                                        .justify_end()
                                                        .gap(spacing::MD)
                                                        .child(
                                                            Button::new("edit")
                                                                .ghost()
                                                                .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                                    // TODO: Navigate to file selection page
                                                                    log::info!("Edit clicked");
                                                                }))
                                                                .child("Edit"),
                                                        )
                                                        .child(
                                                            Button::new("add")
                                                                .with_variant(gpui_component::button::ButtonVariant::Primary)
                                                                .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                                    // TODO: Add more files
                                                                    log::info!("Add clicked");
                                                                }))
                                                                .child("Add"),
                                                        ),
                                                ),
                                        ),
                                )
                            }),
                    )
                    .child(
                        // Nearby devices section
                        v_flex()
                            .gap(spacing::MD)
                            .w_full()
                            .child(
                                // Header with action buttons
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
                                            .gap(spacing::SM)
                                            .child(
                                                // Scan button
                                                Button::new("scan")
                                                    .ghost()
                                                    .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                        // TODO: Trigger device scan
                                                        log::info!("Scan clicked");
                                                    }))
                                                    .child("🔄"),
                                            )
                                            .child(
                                                // Manual address button
                                                Button::new("manual")
                                                    .ghost()
                                                    .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                        // TODO: Show manual address dialog
                                                        log::info!("Manual address clicked");
                                                    }))
                                                    .child("📍"),
                                            )
                                            .child(
                                                // Favorites button
                                                Button::new("favorites")
                                                    .ghost()
                                                    .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                        // TODO: Show favorites dialog
                                                        log::info!("Favorites clicked");
                                                    }))
                                                    .child("⭐"),
                                            )
                                            .child(
                                                // Send mode button
                                                Button::new("send-mode")
                                                    .ghost()
                                                    .on_click(cx.listener(|_this, _event, _window, _cx| {
                                                        // TODO: Show send mode menu
                                                        log::info!("Send mode clicked");
                                                    }))
                                                    .child("⚙️"),
                                            ),
                                    ),
                            )
                            .child(
                                // Device list
                                v_flex()
                                    .gap(spacing::SM)
                                    .w_full()
                                    .child(
                                        // Placeholder when no devices
                                        div()
                                            .w_full()
                                            .p(spacing::LG)
                                            .bg(cx.theme().muted)
                                            .rounded_lg()
                                            .opacity(30.0)
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .text_center()
                                                    .child("Scanning for devices..."),
                                            ),
                                    )
                                    // TODO: Render actual devices from device_state
                                    // .children(devices.iter().map(|device| {
                                    //     DeviceCard::new(device.clone())
                                    //         .on_select(|device, window, cx| {
                                    //         // Handle device selection
                                    //     })
                                    // })),
                            ),
                    )
                    .child(
                        // Help text
                        div()
                            .w_full()
                            .p(spacing::MD)
                            .text_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Select files and choose a nearby device to send"),
                            ),
                    )
                    .child(
                        // Bottom spacing
                        div().h(px(50.)),
                    ),
            )
    }
}
