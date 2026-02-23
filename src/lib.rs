use gpui::{hsla, px, size, App, AppContext, Application, Bounds, WindowBounds, WindowOptions};
use gpui_component::theme::Theme;
use gpui_component::Anchor;
use gpui_component::Root;
use gpui_component_assets::Assets as ComponentAssets;

use log::LevelFilter;
use ohos_hilog_binding::log::Config;
use openharmony_ability::OpenHarmonyApp;

mod app;
mod assets;
mod core;
mod platform;
mod state;
mod ui;

#[openharmony_ability_derive::ability]
pub fn openharmony_app(app: OpenHarmonyApp) {
    ohos_hilog_binding::log::init_once(Config::default().with_max_level(LevelFilter::Debug));
    platform::clipboard::set_ohos_app(app.clone());

    let inner_app = app.clone();
    // Initialize and run GPUI application
    // The event loop is automatically integrated by the platform
    Application::with_platform(gpui_platform::current_platform(false))
        .with_assets(assets::NearSendAssets(ComponentAssets))
        .with_ohos_app(app.clone())
        .run(move |cx: &mut App| {
            gpui_component::init(cx);
            gpui_router::init(cx);
            Theme::global_mut(cx).overlay = hsla(0.0, 0.0, 0.0, 0.58);
            Theme::global_mut(cx).notification.placement = Anchor::BottomCenter;
            Theme::global_mut(cx).notification.margins.bottom = px(56.);

            // Create a shared tokio runtime on a background thread.
            // All async work (server, transfers, discovery) goes through this handle.
            let tokio_handle = {
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .worker_threads(2)
                        .thread_name("near-send-tokio")
                        .build()
                        .expect("Failed to create tokio runtime");
                    tx.send(rt.handle().clone()).unwrap();
                    // Keep the runtime alive for the lifetime of the app
                    rt.block_on(std::future::pending::<()>());
                });
                rx.recv().expect("Failed to receive tokio handle")
            };

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
                        let server = cx.new(|_| core::server::ServerManager::new(53317));

                        // Generate self-signed cert for TLS
                        let cert = match core::cert::generate_self_signed_cert() {
                            Ok(cert) => {
                                log::info!("Generated self-signed TLS certificate");
                                Some(cert)
                            }
                            Err(e) => {
                                log::error!("Failed to generate TLS certificate: {}", e);
                                None
                            }
                        };

                        let app_state = cx.new(|_| {
                            let mut state = state::app_state::AppState::new(
                                server.clone(),
                                tokio_handle.clone(),
                            );
                            state.cert = cert;
                            state
                        });
                        let device_state = cx.new(|_| state::device_state::DeviceState::new());
                        let transfer_state =
                            cx.new(|_| state::transfer_state::TransferState::new());
                        let history_state = cx.new(|_| state::history_state::HistoryState::new());
                        app::AppRoot::new(
                            cx,
                            app_state,
                            device_state,
                            transfer_state,
                            history_state,
                        )
                    });
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .unwrap();
            cx.activate(true);
        });
}
