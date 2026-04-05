use synforge_worker::WorkerRuntime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    synforge_core::logging::init_tracing();
    let runtime = WorkerRuntime::new();
    runtime.run_from_env().await?;
    Ok(())
}
