mod artifact;
mod executor;
mod git;
mod logging;
mod meson;
mod protocol;
mod spec;

use std::time::Duration;

use synforge_core::{
    error::SynforgeError,
    model::{WorkerAction, WorkerBuildResult, WorkerJobPayload, WorkerResult},
};

pub use executor::{BuildExecutor, RpmBuildExecutor};
pub use protocol::WorkerTransportHandle;

#[derive(Debug, Default, Clone)]
pub struct WorkerRuntime<E = RpmBuildExecutor> {
    executor: E,
}

impl WorkerRuntime<RpmBuildExecutor> {
    pub fn new() -> Self {
        Self {
            executor: RpmBuildExecutor,
        }
    }
}

impl<E> WorkerRuntime<E>
where
    E: BuildExecutor,
{
    pub async fn run_from_env(&self) -> anyhow::Result<WorkerResult> {
        let worker_id = env_required("SYNFORGE_WORKER_ID")?;
        let connect_addr =
            env_string("SYNFORGE_WORKER_CONNECT_ADDR").unwrap_or_else(|| "daemon:8090".to_string());
        let socket_timeout = env_u64("SYNFORGE_WORKER_SOCKET_TIMEOUT_SECONDS").unwrap_or(30);
        let transport = WorkerTransportHandle::connect(
            &connect_addr,
            &worker_id,
            std::time::Duration::from_secs(socket_timeout),
        )
        .await?;
        let payload = transport.receive_assignment().await?;
        self.run_with_transport(payload, Some(transport)).await
    }

    pub async fn run(&self, payload: WorkerJobPayload) -> anyhow::Result<WorkerResult> {
        self.run_with_transport(payload, None).await
    }

    async fn run_with_transport(
        &self,
        payload: WorkerJobPayload,
        transport: Option<WorkerTransportHandle>,
    ) -> anyhow::Result<WorkerResult> {
        let heartbeat_task = transport.as_ref().map(|transport| {
            let transport = transport.clone();
            tokio::spawn(async move {
                let interval = Duration::from_secs(10);
                loop {
                    tokio::time::sleep(interval).await;
                    if let Err(error) = transport.send_heartbeat().await {
                        tracing::warn!("failed to send worker heartbeat: {}", error);
                        break;
                    }
                }
            })
        });
        tokio::fs::create_dir_all(&payload.workspace_dir).await?;
        let local_result = match &payload.action {
            WorkerAction::Parse(parse) => {
                WorkerResult::Parse(spec::execute_spec_parse(&payload, parse).await?)
            }
            WorkerAction::Build(build) => {
                build
                    .package
                    .validate()
                    .map_err(|error: SynforgeError| anyhow::anyhow!(error.to_string()))?;
                WorkerResult::Build(
                    self.executor
                        .execute_build(&build.package, &payload, build, transport.clone())
                        .await?,
                )
            }
        };
        if let Some(task) = heartbeat_task {
            task.abort();
            let _ = task.await;
        }
        publish_worker_result(
            transport.as_ref(),
            &local_result,
            &payload.workspace_dir.join("rpmbuild"),
        )
        .await?;
        Ok(local_result)
    }
}

fn env_required(name: &str) -> anyhow::Result<String> {
    std::env::var(name).map_err(|_| anyhow::anyhow!("missing required env var {}", name))
}

fn env_string(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn env_u64(name: &str) -> Option<u64> {
    std::env::var(name).ok()?.trim().parse::<u64>().ok()
}

async fn publish_worker_result(
    transport: Option<&WorkerTransportHandle>,
    result: &WorkerResult,
    artifact_root: &std::path::Path,
) -> anyhow::Result<()> {
    let Some(transport) = transport else {
        return Ok(());
    };

    let uploaded_result = match result {
        WorkerResult::Parse(parse) => WorkerResult::Parse(parse.clone()),
        WorkerResult::Build(build) => WorkerResult::Build(WorkerBuildResult {
            artifacts: {
                for artifact in &build.artifacts {
                    transport.send_artifact(artifact_root, artifact).await?;
                }
                Vec::new()
            },
            ..build.clone()
        }),
    };
    transport.send_result(uploaded_result).await?;
    Ok(())
}
