use synforge_core::constants::{
    DEFAULT_DAEMON_WORKER_SOCKET_PORT, DEFAULT_WORKER_SOCKET_TIMEOUT_SECONDS,
};
use synforge_worker::WorkerRuntime;

struct WorkerArgs {
    worker_id: String,
    connect_addr: String,
    socket_timeout_seconds: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    synforge_core::logging::init_tracing();
    let args = parse_args(std::env::args().skip(1).collect())?;
    let runtime = WorkerRuntime::new();
    runtime
        .run_remote(
            &args.worker_id,
            &args.connect_addr,
            args.socket_timeout_seconds,
        )
        .await?;
    Ok(())
}

fn parse_args(args: Vec<String>) -> anyhow::Result<WorkerArgs> {
    let mut worker_id = None;
    let mut connect_addr = format!("daemon:{}", DEFAULT_DAEMON_WORKER_SOCKET_PORT);
    let mut socket_timeout_seconds = DEFAULT_WORKER_SOCKET_TIMEOUT_SECONDS;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--worker-id" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("missing value for --worker-id"))?;
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return Err(anyhow::anyhow!("--worker-id must not be empty"));
                }
                worker_id = Some(trimmed.to_string());
            }
            "--connect-addr" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("missing value for --connect-addr"))?;
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return Err(anyhow::anyhow!("--connect-addr must not be empty"));
                }
                connect_addr = trimmed.to_string();
            }
            "--socket-timeout-seconds" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("missing value for --socket-timeout-seconds"))?;
                socket_timeout_seconds = value.trim().parse::<u64>().map_err(|error| {
                    anyhow::anyhow!(
                        "invalid value for --socket-timeout-seconds '{}': {}",
                        value,
                        error
                    )
                })?;
            }
            "--help" | "-h" => {
                println!(
                    "Usage: worker --worker-id <id> [--connect-addr <host:port>] [--socket-timeout-seconds <seconds>]"
                );
                std::process::exit(0);
            }
            _ => {
                return Err(anyhow::anyhow!("unknown argument: {arg}"));
            }
        }
    }

    let worker_id =
        worker_id.ok_or_else(|| anyhow::anyhow!("missing required --worker-id argument"))?;

    if socket_timeout_seconds == 0 {
        return Err(anyhow::anyhow!(
            "--socket-timeout-seconds must be greater than zero"
        ));
    }

    Ok(WorkerArgs {
        worker_id,
        connect_addr,
        socket_timeout_seconds,
    })
}
