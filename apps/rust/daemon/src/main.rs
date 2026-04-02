use std::sync::Arc;

use anyhow::Context;
use synforge_core::DaemonConfig;
use synforge_orchestrator::SynforgeService;
use tracing::warn;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,synforge=debug".into()),
        )
        .init();

    let config = DaemonConfig::load_from_env()?;
    let service = SynforgeService::new(config.clone()).await?;
    let listener = tokio::net::TcpListener::bind(&config.listen_addr)
        .await
        .with_context(|| format!("failed to bind {}", config.listen_addr))?;
    let app = synforge_serve::router(Arc::clone(&service));
    tracing::info!("daemon listening on {}", config.listen_addr);
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
