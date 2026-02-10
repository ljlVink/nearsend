// Async initialization utilities for OpenHarmony compatibility
// These functions should be called from GPUI async context (cx.spawn)

use crate::core::{discovery::DiscoveryService, server::ServerManager};
use gpui::AsyncApp;
use localsend::http::state::ClientInfo;
use tokio::runtime::Handle;

/// Initialize discovery service in async context
pub async fn init_discovery(
    discovery: &mut DiscoveryService,
    _cx: &mut AsyncApp,
) -> anyhow::Result<()> {
    discovery.start_sync();
    log::info!("Discovery service started");
    Ok(())
}

/// Initialize server in async context
pub async fn init_server(
    server: &mut ServerManager,
    client_info: ClientInfo,
    use_https: bool,
    handle: &Handle,
    _cx: &mut AsyncApp,
) -> anyhow::Result<()> {
    server.start(client_info, use_https, handle)?;
    log::info!("Server started");
    Ok(())
}
