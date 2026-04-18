use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use synforge_core::config::DaemonConfig;
use synforge_core::constants::DEFAULT_WEBUI_STATIC_DIR;
use synforge_daemon::SynforgeService;
use tracing::warn;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    synforge_core::logging::init_tracing();

    let config = DaemonConfig::load()?;
    let service = SynforgeService::new(config).await?;
    let listen_addr = service.config().listen_addr.clone();
    let listener = tokio::net::TcpListener::bind(&listen_addr)
        .await
        .with_context(|| format!("failed to bind {listen_addr}"))?;
    let app = synforge_daemon::router(
        Arc::clone(&service),
        PathBuf::from(DEFAULT_WEBUI_STATIC_DIR),
    );
    tracing::info!("daemon listening on {}", listen_addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(Arc::clone(&service)))
        .await?;
    Ok(())
}

async fn shutdown_signal(service: Arc<SynforgeService>) {
    #[cfg(unix)]
    let terminate = async {
        let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler");
        signal.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        result = tokio::signal::ctrl_c() => {
            if let Err(error) = result {
                warn!("failed to listen for Ctrl-C: {}", error);
            }
        }
        _ = terminate => {}
    }

    service.graceful_shutdown().await;
}
