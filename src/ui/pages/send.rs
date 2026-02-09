use crate::state::device_state::DeviceState;
use crate::ui::components::{device_card::DeviceCard, file_list::FileList};
use gpui::{
    div, prelude::*, px, Context, Entity, SharedString, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    scroll::ScrollableElement,
    v_flex, ActiveTheme as _, StyledExt as _,
};
use localsend::http::state::ClientInfo;
use std::path::PathBuf;
use crate::ui::theme::{spacing, sizing};

/// Send page for selecting files and devices (mobile-first design)
pub struct SendPage {
    device_state: Entity<DeviceState>,
    selected_files: Vec<PathBuf>,
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

        v_flex()
            .size_full()
            .p(spacing::MD)
            .gap(spacing::LG)
            .bg(cx.theme().background)
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
                    .child(
                        Button::new("Choose Files")
                            .primary()
                            .w_full()
                            .h(sizing::BUTTON_HEIGHT)
                            .on_click(cx.listener(|_this, _event, _window, _cx| {
                                // TODO: Implement file picker
                                log::info!("File picker clicked");
                            })),
                    )
                    .when(!selected_files.is_empty(), |this| {
                        // TODO: Render FileList - need to create view properly
                        this.child(div().child(format!("{} files selected", selected_files.len())))
                    }),
            )
            .child(
                // Device list section
                v_flex()
                    .gap(spacing::MD)
                    .w_full()
                    .flex_1()
                    .child(
                        div()
                            .text_lg()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("Nearby Devices"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .child(
                                // TODO: Fetch devices from device_state and render DeviceCard for each
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .p(spacing::LG)
                                    .text_center()
                                    .child("Scanning for devices..."),
                            ),
                    ),
            )
    }
}
