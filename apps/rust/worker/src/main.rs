use synforge_worker::WorkerRuntime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,synforge=debug".into()),
        )
        .init();
    let runtime = WorkerRuntime::new();
    runtime.run_from_env().await?;
    Ok(())
}
