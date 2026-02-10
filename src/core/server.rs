use localsend::http::server::start_with_port;
use localsend::http::state::ClientInfo;
use std::path::PathBuf;
use tokio::runtime::Handle;
use tokio::sync::oneshot;

/// HTTP server manager using LocalSend core
pub struct ServerManager {
    port: u16,
    save_directory: PathBuf,
    stop_tx: Option<oneshot::Sender<()>>,
}

impl ServerManager {
    pub fn new(port: u16, save_directory: PathBuf) -> Self {
        Self {
            port,
            save_directory,
            stop_tx: None,
        }
    }

    /// Start the HTTP server on the shared tokio runtime.
    pub fn start(
        &mut self,
        client_info: ClientInfo,
        use_https: bool,
        handle: &Handle,
    ) -> anyhow::Result<()> {
        let (stop_tx, stop_rx) = oneshot::channel();
        self.stop_tx = Some(stop_tx);

        let tls_config = if use_https { None } else { None };
        let port = self.port;

        handle.spawn(async move {
            if let Err(e) = start_with_port(port, tls_config, client_info, true, stop_rx).await {
                log::error!("Server error: {}", e);
            }
        });

        log::info!(
            "Starting HTTP server on port {} (HTTPS: {})",
            self.port,
            use_https
        );
        Ok(())
    }

    /// Stop the server
    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
            log::info!("Stopping HTTP server");
        }
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.stop_tx.is_some()
    }
}
