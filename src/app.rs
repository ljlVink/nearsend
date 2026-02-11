//! App entry: router root. Uses gpui-router to show Home (tabs), History,
//! Progress, or Selected Files based on pathname.

use crate::state::{
    app_state::AppState, device_state::DeviceState, history_state::HistoryState,
    send_selection_state::SendSelectionState,
    transfer_state::TransferState,
};
use crate::ui::pages::{HistoryPage, HomePage, ProgressPage, SelectedFilesPage};
use gpui::{div, prelude::*, Context, Entity, IntoElement, Window};
use gpui_component::{v_flex, ActiveTheme as _, Root};
use gpui_router::RouterState;

/// Router root: shows Home (tabs), History, Progress, or Selected Files.
pub struct AppRoot {
    home_entity: Entity<HomePage>,
    history_entity: Entity<HistoryPage>,
    progress_entity: Entity<ProgressPage>,
    selected_files_entity: Entity<SelectedFilesPage>,
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
        let home_entity = cx.new(|_| {
            HomePage::new(
                app_state,
                device_state,
                transfer_state.clone(),
                send_selection_state.clone(),
            )
        });
        let history_entity = cx.new(|_| HistoryPage::new().with_history_state(history_state));
        let progress_entity = cx.new(|_| {
            ProgressPage::new(
                transfer_state,
                crate::state::transfer_state::TransferDirection::Send,
            )
        });
        let selected_files_entity =
            cx.new(|_| SelectedFilesPage::new(send_selection_state.clone()));
        Self {
            home_entity,
            history_entity,
            progress_entity,
            selected_files_entity,
        }
    }
}

impl gpui::Render for AppRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        let pathname = RouterState::global(cx).location.pathname.clone();
        let content = match pathname.as_ref() {
            "/receive/history" => self.history_entity.clone().into_any_element(),
            "/transfer/progress" => self.progress_entity.clone().into_any_element(),
            "/send/files" => self.selected_files_entity.clone().into_any_element(),
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
