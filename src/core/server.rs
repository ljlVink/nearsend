use localsend::http::server::start_with_port;
use localsend::http::state::ClientInfo;
use std::path::PathBuf;
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

    /// Start the HTTP server
    ///
    /// Note: This uses tokio::spawn internally, which requires tokio runtime.
    /// For OpenHarmony, ensure tokio runtime is available or use GPUI async context.
    pub async fn start(&mut self, client_info: ClientInfo, use_https: bool) -> anyhow::Result<()> {
        let (stop_tx, stop_rx) = oneshot::channel();
        self.stop_tx = Some(stop_tx);

        let tls_config = if use_https {
            // TODO: Generate TLS certificate using localsend::crypto
            None // Placeholder - need to implement TLS cert generation
        } else {
            None
        };

        // Note: localsend::http::server::start_with_port uses tokio internally
        // For OpenHarmony, this should work if tokio runtime is available
        // If there are issues, we may need to wrap it in GPUI async context
        let port = self.port;
        let client_info_clone = client_info.clone();

        // Start server - localsend core uses tokio internally
        // This should work on OpenHarmony if tokio runtime is properly configured
        tokio::spawn(async move {
            if let Err(e) =
                start_with_port(port, tls_config, client_info_clone, true, stop_rx).await
            {
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
    pub async fn stop(&mut self) {
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
