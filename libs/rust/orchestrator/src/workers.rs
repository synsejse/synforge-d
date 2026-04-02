use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, LogsOptionsBuilder, RemoveContainerOptionsBuilder,
};
use bollard::Docker;
use futures_util::StreamExt;
use synforge_core::{DaemonConfig, WorkerJobPayload, WorkerResult};
use tokio::io::AsyncWriteExt;

use crate::job_lifecycle::JobLifecycle;
use crate::sessions::WorkerSessionBroker;

#[derive(Debug, Clone)]
pub struct WorkerExecution {
    pub result: WorkerResult,
    pub logs_path: Option<PathBuf>,
}

#[async_trait]
pub trait WorkerLauncher: Send + Sync {
    async fn run_job(
        &self,
        payload: &WorkerJobPayload,
        config: &DaemonConfig,
    ) -> anyhow::Result<WorkerExecution>;

    async fn shutdown(&self) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct DockerWorkerLauncher {
    docker: Docker,
    sessions: WorkerSessionBroker,
    lifecycle: Arc<JobLifecycle>,
}

impl DockerWorkerLauncher {
    pub fn new(sessions: WorkerSessionBroker, lifecycle: Arc<JobLifecycle>) -> anyhow::Result<Self> {
        Ok(Self {
            docker: Docker::connect_with_local_defaults()?,
            sessions,
            lifecycle,
        })
    }
}

#[async_trait]
impl WorkerLauncher for DockerWorkerLauncher {
    async fn run_job(
        &self,
        payload: &WorkerJobPayload,
        config: &DaemonConfig,
    ) -> anyhow::Result<WorkerExecution> {
        let paths = config.runtime_paths();
        tokio::fs::create_dir_all(&payload.workspace_dir).await?;
        tokio::fs::create_dir_all(paths.job_logs_dir(payload.job_id)).await?;
        let logs_path = paths.job_worker_log_path(payload.job_id);
        let session = self
            .sessions
            .create_session(payload.job_id, payload.clone())
            .await?;

        let container_name = format!("synforge-worker-{}", payload.job_id);
        let container = self
            .docker
            .create_container(
                Some(
                    CreateContainerOptionsBuilder::default()
                        .name(container_name.as_str())
                        .build(),
                ),
                ContainerCreateBody {
                    image: Some(config.worker_image.clone()),
                    env: Some(worker_env(payload, config, &session.worker_id)),
                    host_config: Some(HostConfig {
                        auto_remove: Some(true),
                        privileged: Some(true),
                        extra_hosts: Some(vec!["host.docker.internal:host-gateway".to_string()]),
                        ..Default::default()
                    }),
                    working_dir: Some("/synforge".to_string()),
                    ..Default::default()
                },
            )
            .await
            .with_context(|| format!("failed to create worker container {}", container_name))?;

        let container_id = container.id;
        self.sessions
            .set_container_id(payload.job_id, container_id.clone())
            .await;
        self.docker
            .start_container(&container_id, None::<bollard::query_parameters::StartContainerOptions>)
            .await
            .with_context(|| format!("failed to start worker container {}", container_id))?;
        self.lifecycle.mark_running(payload.job_id, &container_id).await?;

        let mut log_file = tokio::fs::File::create(&logs_path).await?;
        let mut logs = self.docker.logs(
            &container_id,
            Some(
                LogsOptionsBuilder::default()
                    .follow(true)
                    .stdout(true)
                    .stderr(true)
                    .timestamps(false)
                    .tail("all")
                    .build(),
            ),
        );
        while let Some(item) = logs.next().await {
            let item = item?;
            log_file.write_all(item.into_bytes().as_ref()).await?;
        }

        let mut wait = self
            .docker
            .wait_container(&container_id, None::<bollard::query_parameters::WaitContainerOptions>);
        while let Some(next) = wait.next().await {
            next?;
        }

        let result = self
            .sessions
            .wait_for_result(payload.job_id, Duration::from_secs(10))
            .await?
            .ok_or_else(|| anyhow::anyhow!("worker {} exited without uploading a result", payload.job_id))?;

        self.docker
            .remove_container(
                &container_id,
                Some(RemoveContainerOptionsBuilder::default().force(true).build()),
            )
            .await
            .ok();
        self.sessions.remove_session(payload.job_id);

        Ok(WorkerExecution {
            result,
            logs_path: Some(logs_path),
        })
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        for session in self.sessions.active_container_sessions().await {
            self.docker
                .stop_container(
                    &session.container_id,
                    None::<bollard::query_parameters::StopContainerOptions>,
                )
                .await
                .ok();
            self.docker
                .remove_container(
                    &session.container_id,
                    Some(RemoveContainerOptionsBuilder::default().force(true).build()),
                )
                .await
                .ok();
            self.sessions.remove_session(session.job_id);
        }
        Ok(())
    }
}

fn worker_env(payload: &WorkerJobPayload, config: &DaemonConfig, worker_id: &str) -> Vec<String> {
    vec![
        format!("SYNFORGE_WORKER_ID={worker_id}"),
        format!(
            "SYNFORGE_WORKER_CONNECT_ADDR={}",
            config.worker_connect_addr.trim()
        ),
        format!("SYNFORGE_JOB_ID={}", payload.job_id),
    ]
}
