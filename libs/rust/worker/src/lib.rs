mod artifact;
mod executor;
mod git;
mod logging;
mod meson;
mod protocol;
mod spec;

use std::time::Duration;

use synforge_core::{
    constants::DEFAULT_WORKER_HEARTBEAT_INTERVAL_SECONDS,
    error::SynforgeError,
    model::{WorkerAction, WorkerBuildResult, WorkerJobPayload, WorkerResult},
    validation::{PackageDefinitionValidator, Validator},
};
use tracing::{info, warn};

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
    pub async fn run_remote(
        &self,
        worker_id: &str,
        connect_addr: &str,
        socket_timeout_seconds: u64,
    ) -> anyhow::Result<WorkerResult> {
        info!(
            worker_id,
            connect_addr, socket_timeout_seconds, "connecting worker runtime to daemon"
        );
        let transport = WorkerTransportHandle::connect(
            connect_addr,
            worker_id,
            std::time::Duration::from_secs(socket_timeout_seconds),
        )
        .await?;
        info!(worker_id, "worker connected; waiting for assignment");
        let payload = transport.receive_assignment().await?;
        let (action, package_name, mock_chroot) = action_descriptor(&payload.action);
        info!(
            worker_id,
            job_id = %payload.job_id,
            action,
            package_name,
            mock_chroot,
            timeout_seconds = payload.timeout_seconds,
            "worker assignment received"
        );
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
        let (action, package_name, mock_chroot) = action_descriptor(&payload.action);
        info!(
            job_id = %payload.job_id,
            action,
            package_name,
            mock_chroot,
            workspace = %payload.workspace_dir.display(),
            timeout_seconds = payload.timeout_seconds,
            "worker job started"
        );
        let heartbeat_task = transport.as_ref().map(|transport| {
            let transport = transport.clone();
            tokio::spawn(async move {
                let interval = Duration::from_secs(DEFAULT_WORKER_HEARTBEAT_INTERVAL_SECONDS);
                loop {
                    tokio::time::sleep(interval).await;
                    if let Err(error) = transport.send_heartbeat().await {
                        warn!("failed to send worker heartbeat: {}", error);
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
                PackageDefinitionValidator
                    .validate(&build.package)
                    .map_err(|error: SynforgeError| anyhow::anyhow!(error.to_string()))?;
                WorkerResult::Build(
                    self.executor
                        .execute_build(&build.package, &payload, build, transport.clone())
                        .await?,
                )
            }
        };
        match &local_result {
            WorkerResult::Parse(parse_result) => info!(
                job_id = %payload.job_id,
                package_name = %parse_result.parsed.name,
                revision = %parse_result.revision.comparison_key(),
                "worker parse completed"
            ),
            WorkerResult::Build(build_result) => info!(
                job_id = %payload.job_id,
                package_name = %build_result.package_name,
                status = ?build_result.status,
                artifact_count = build_result.artifacts.len(),
                "worker build completed"
            ),
        }
        if let Some(task) = heartbeat_task {
            task.abort();
            if let Err(error) = task.await
                && !error.is_cancelled()
            {
                warn!(?error, "worker heartbeat task panicked");
            }
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
                info!(
                    job_id = %build.job_id,
                    artifact_count = build.artifacts.len(),
                    "uploading build artifacts to daemon"
                );
                for artifact in &build.artifacts {
                    transport.send_artifact(artifact_root, artifact).await?;
                }
                Vec::new()
            },
            ..build.clone()
        }),
    };
    info!("uploading worker result to daemon");
    transport.send_result(uploaded_result).await?;
    Ok(())
}

fn action_descriptor(action: &WorkerAction) -> (&'static str, &str, &str) {
    match action {
        WorkerAction::Parse(parse) => ("parse", parse.package_name.as_str(), "-"),
        WorkerAction::Build(build) => ("build", build.package_name.as_str(), &build.mock_chroot),
    }
}
