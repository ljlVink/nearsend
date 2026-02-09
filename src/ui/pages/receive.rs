use crate::state::transfer_state::TransferState;
use crate::ui::components::transfer_item::TransferItem;
use crate::ui::theme::{sizing, spacing};
use gpui::{div, prelude::*, px, Context, Entity, SharedString, Window};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    tab::{Tab, TabBar},
    v_flex, ActiveTheme as _, Sizable as _, StyledExt as _,
};

/// Quick Save mode
#[derive(Clone, Copy, PartialEq, Eq)]
enum QuickSaveMode {
    Off,
    Favorites,
    On,
}

/// Receive page for showing received files and history (mobile-first design)
/// Aligned with LocalSend mobile UI
pub struct ReceivePage {
    transfer_state: Entity<TransferState>,
    quick_save_mode: QuickSaveMode,
    show_advanced: bool,
    server_alias: String,
    server_ip: String,
    server_port: u16,
    server_running: bool,
}

impl ReceivePage {
    pub fn new(transfer_state: Entity<TransferState>) -> Self {
        Self {
            transfer_state,
            quick_save_mode: QuickSaveMode::Off,
            show_advanced: false,
            server_alias: "NearSend".to_string(),
            server_ip: "192.168.1.100".to_string(), // TODO: Get from actual server state
            server_port: 53317,
            server_running: false, // TODO: Get from actual server state
        }
    }
}

impl gpui::Render for ReceivePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let transfer_state = self.transfer_state.clone();
        let show_advanced = self.show_advanced;
        let quick_save_mode = self.quick_save_mode;
        let server_alias = self.server_alias.clone();
        let server_ip = self.server_ip.clone();
        let server_port = self.server_port;
        let server_running = self.server_running;

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .relative()
            .child(
                // Main content area
                div().flex_1().w_full().overflow_y_scrollbar().child(
                    v_flex()
                        .w_full()
                        .p(spacing::LG)
                        .gap(spacing::XL)
                        .items_center()
                        .child(
                            // Logo and device info section (centered)
                            v_flex()
                                .flex_1()
                                .items_center()
                                .justify_center()
                                .gap(spacing::MD)
                                .child(
                                    // Logo placeholder (TODO: Add actual logo image)
                                    div()
                                        .w(px(120.))
                                        .h(px(120.))
                                        .rounded_full()
                                        .bg(cx.theme().primary)
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            div()
                                                .text_2xl()
                                                .font_bold()
                                                .text_color(cx.theme().primary_foreground)
                                                .child("NS"),
                                        ),
                                )
                                .child(
                                    // Device alias
                                    div()
                                        .text_2xl()
                                        .font_bold()
                                        .text_color(cx.theme().foreground)
                                        .text_center()
                                        .child(server_alias.clone()),
                                )
                                .child(
                                    // IP address or offline status
                                    div()
                                        .text_2xl()
                                        .text_color(cx.theme().muted_foreground)
                                        .text_center()
                                        .child(if server_running {
                                            format!("#{}", server_ip)
                                        } else {
                                            "Offline".to_string()
                                        }),
                                ),
                        )
                        .child(
                            // Quick Save section
                            v_flex()
                                .gap(spacing::SM)
                                .items_center()
                                .child(
                                    div()
                                        .text_base()
                                        .text_color(cx.theme().foreground)
                                        .child("Quick Save"),
                                )
                                .child(
                                    // Quick Save segmented button (using TabBar as segmented control)
                                    TabBar::new("quick-save")
                                        .segmented()
                                        .selected_index(match quick_save_mode {
                                            QuickSaveMode::Off => 0,
                                            QuickSaveMode::Favorites => 1,
                                            QuickSaveMode::On => 2,
                                        })
                                        .on_click(cx.listener(move |this, index, _window, _cx| {
                                            this.quick_save_mode = match *index {
                                                0 => QuickSaveMode::Off,
                                                1 => QuickSaveMode::Favorites,
                                                2 => QuickSaveMode::On,
                                                _ => QuickSaveMode::Off,
                                            };
                                        }))
                                        .children([
                                            Tab::new().label("Off"),
                                            Tab::new().label("Favorites"),
                                            Tab::new().label("On"),
                                        ]),
                                ),
                        )
                        .child(
                            // Spacing
                            div().h(px(15.)),
                        ),
                ),
            )
            .child(
                // Corner buttons (top right)
                div().absolute().top(spacing::MD).right(spacing::MD).child(
                    h_flex()
                        .gap(spacing::SM)
                        .child(
                            // History button (only show when not advanced)
                            div().when(!show_advanced, |this| {
                                this.child(
                                    Button::new("history")
                                        .ghost()
                                        .on_click(cx.listener(|_this, _event, _window, _cx| {
                                            // TODO: Navigate to history page
                                            log::info!("History clicked");
                                        }))
                                        .child("📜"),
                                )
                            }),
                        )
                        .child(
                            // Info button
                            Button::new("info")
                                .ghost()
                                .on_click(cx.listener(move |this, _event, _window, _cx| {
                                    this.show_advanced = !this.show_advanced;
                                }))
                                .child("ℹ️"),
                        ),
                ),
            )
            .child(
                // Advanced info box (top right, animated)
                div()
                    .absolute()
                    .top(spacing::MD)
                    .right(spacing::MD)
                    .when(show_advanced, |this| {
                        this.child(
                            div()
                                .bg(cx.theme().secondary)
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_lg()
                                .p(spacing::MD)
                                .shadow_lg()
                                .w(px(250.))
                                .child(
                                    v_flex()
                                        .gap(spacing::SM)
                                        .child(
                                            // Alias row
                                            h_flex()
                                                .justify_between()
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
                                                        .child(server_alias.clone()),
                                                ),
                                        )
                                        .child(
                                            // IP row
                                            h_flex()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child("IP:"),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(cx.theme().foreground)
                                                        .child(server_ip.clone()),
                                                ),
                                        )
                                        .child(
                                            // Port row
                                            h_flex()
                                                .justify_between()
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
                        )
                    }),
            )
    }
}
