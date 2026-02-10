use crate::core::cert::CertPair;
use localsend::http::server::{self, TlsConfig};
use localsend::http::state::ClientInfo;
use tokio::runtime::Handle;
use tokio::sync::oneshot;

/// Server manager that delegates to localsend core server.
pub struct ServerManager {
    port: u16,
    stop_tx: Option<oneshot::Sender<()>>,
}

impl ServerManager {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            stop_tx: None,
        }
    }

    pub fn start(
        &mut self,
        client_info: ClientInfo,
        use_https: bool,
        cert: Option<CertPair>,
        handle: &Handle,
    ) -> anyhow::Result<()> {
        if self.is_running() {
            log::warn!("Server already running on port {}", self.port);
            return Ok(());
        }

        let (stop_tx, stop_rx) = oneshot::channel();
        self.stop_tx = Some(stop_tx);

        let tls_config = if use_https {
            cert.map(|c| TlsConfig {
                cert: c.cert_pem,
                private_key: c.private_key_pem,
            })
        } else {
            None
        };

        let port = self.port;
        handle.spawn(async move {
            if let Err(e) = server::start_with_port(port, tls_config, client_info, true, stop_rx).await {
                log::error!("Server error: {}", e);
            }
        });

        log::info!(
            "Starting LocalSend server on port {} (https={})",
            self.port,
            use_https
        );

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
            log::info!("Stopping LocalSend server");
        }
    }

    pub fn is_running(&self) -> bool {
        self.stop_tx.is_some()
    }
}
