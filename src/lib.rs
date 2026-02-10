use gpui::{px, size, App, AppContext, Application, Bounds, WindowBounds, WindowOptions};
use gpui_component::Root;
use gpui_component_assets::Assets as ComponentAssets;

use log::LevelFilter;
use ohos_hilog_binding::log::Config;
use openharmony_ability::OpenHarmonyApp;

mod app;
mod assets;
mod core;
mod state;
mod ui;

// Legacy modules (to be removed after migration)
// These modules are commented out as they use dependencies not in Cargo.toml
// They will be replaced by the localsend core implementation
// mod client;
// mod discovery;
// mod protocol;
// mod server;

#[openharmony_ability_derive::ability]
pub fn openharmony_app(app: OpenHarmonyApp) {
    ohos_hilog_binding::log::init_once(Config::default().with_max_level(LevelFilter::Debug));

    let inner_app = app.clone();
    // Initialize and run GPUI application
    // The event loop is automatically integrated by the platform
    Application::new()
        .with_assets(assets::NearSendAssets(ComponentAssets))
        .with_ohos_app(app.clone())
        .run(move |cx: &mut App| {
            gpui_component::init(cx);
            gpui_router::init(cx);
            let info = inner_app.content_rect();
            let default_size = size(px(info.width as _), px(info.height as _));
            let bounds = Bounds::centered(None, default_size, cx);

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| {
                        let discovery = cx.new(|_| core::discovery::DiscoveryService::new());
                        let server = cx.new(|_| {
                            core::server::ServerManager::new(
                                53317,
                                std::path::PathBuf::from("./downloads"),
                            )
                        });
                        let transfer = cx.new(|_| core::transfer::TransferService::new());
                        let app_state = cx.new(|_| {
                            state::app_state::AppState::new(
                                discovery.clone(),
                                server.clone(),
                                transfer.clone(),
                            )
                        });
                        let device_state = cx.new(|_| state::device_state::DeviceState::new());
                        let transfer_state =
                            cx.new(|_| state::transfer_state::TransferState::new());
                        let history_state =
                            cx.new(|_| state::history_state::HistoryState::new());
                        app::AppRoot::new(cx, app_state, device_state, transfer_state, history_state)
                    });
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .unwrap();
            cx.activate(true);
        });
}
