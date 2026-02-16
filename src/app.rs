//! App entry: router root. Uses gpui-router to show Home (tabs), History,
//! Progress, or Selected Files based on pathname.

use crate::state::{
    app_state::AppState, device_state::DeviceState, history_state::HistoryState,
    receive_inbox_state::ReceiveInboxState, send_selection_state::SendSelectionState,
    transfer_state::TransferState,
};
use crate::ui::pages::{
    HistoryPage, HomePage, ProgressPage, ReceiveIncomingPage, SelectedFilesPage, WebSendPage,
};
use gpui::{div, prelude::*, Context, Entity, IntoElement, Window};
use gpui_component::{v_flex, ActiveTheme as _, Root};
use gpui_router::RouterState;

/// Router root: shows Home (tabs), History, Progress, or Selected Files.
pub struct AppRoot {
    home_entity: Entity<HomePage>,
    history_entity: Entity<HistoryPage>,
    progress_entity: Entity<ProgressPage>,
    selected_files_entity: Entity<SelectedFilesPage>,
    receive_incoming_entity: Entity<ReceiveIncomingPage>,
    web_send_entity: Entity<WebSendPage>,
    incoming_event_listener_started: bool,
}

impl AppRoot {
    pub fn new(
        cx: &mut Context<Self>,
        app_state: Entity<AppState>,
        device_state: Entity<DeviceState>,
        transfer_state: Entity<TransferState>,
        history_state: Entity<HistoryState>,
    ) -> Self {
        let send_selection_state = cx.new(|_| SendSelectionState::default());
        let receive_inbox_state = cx.new(|_| ReceiveInboxState::default());
        let home_entity = cx.new(|_| {
            HomePage::new(
                app_state.clone(),
                device_state.clone(),
                transfer_state.clone(),
                history_state.clone(),
                send_selection_state.clone(),
                receive_inbox_state.clone(),
            )
        });
        let history_entity = cx.new(|_| {
            HistoryPage::new()
                .with_history_state(history_state)
                .with_receive_inbox_state(receive_inbox_state.clone())
        });
        let progress_entity = cx.new(|_| {
            ProgressPage::new(
                transfer_state,
                crate::state::transfer_state::TransferDirection::Send,
            )
        });
        let selected_files_entity =
            cx.new(|_| SelectedFilesPage::new(app_state.clone(), send_selection_state.clone()));
        let receive_incoming_entity =
            cx.new(|_| ReceiveIncomingPage::new(app_state.clone(), receive_inbox_state));
        let web_send_entity = cx.new(|_| WebSendPage::new(home_entity.clone()));
        Self {
            home_entity,
            history_entity,
            progress_entity,
            selected_files_entity,
            receive_incoming_entity,
            web_send_entity,
            incoming_event_listener_started: false,
        }
    }
}

impl gpui::Render for AppRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.incoming_event_listener_started {
            self.incoming_event_listener_started = true;
            cx.spawn(async move |this, cx| loop {
                crate::core::receive_events::wait_for_incoming_event().await;
                if this.update(cx, |_this, cx| cx.notify()).is_err() {
                    break;
                }
            })
            .detach();
        }

        let _ = self.home_entity.update(cx, |home, cx| {
            home.poll_incoming_events(window, cx);
        });

        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        let pathname = RouterState::global(cx).location.pathname.clone();
        let content = match pathname.as_ref() {
            "/receive/history" => self.history_entity.clone().into_any_element(),
            "/receive/incoming" => self.receive_incoming_entity.clone().into_any_element(),
            "/transfer/progress" => self.progress_entity.clone().into_any_element(),
            "/send/files" => self.selected_files_entity.clone().into_any_element(),
            "/send/link" => self.web_send_entity.clone().into_any_element(),
            _ => self.home_entity.clone().into_any_element(),
        };

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(div().size_full().child(content))
            .children(sheet_layer)
            .children(dialog_layer)
            .children(notification_layer)
    }
}
