use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use bollard::Docker;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::CreateContainerOptionsBuilder;
use futures_util::StreamExt;
use synforge_core::{config::DaemonConfig, model::{WorkerJobPayload, WorkerResult}};
use tracing::warn;

use crate::job_lifecycle::JobLifecycle;
use crate::sessions::WorkerSessionBroker;

#[derive(Clone)]
pub struct DockerWorkerLauncher {
    docker: Docker,
    sessions: WorkerSessionBroker,
    lifecycle: Arc<JobLifecycle>,
    network_mode: Option<String>,
}

impl DockerWorkerLauncher {
    pub async fn new(
        sessions: WorkerSessionBroker,
        lifecycle: Arc<JobLifecycle>,
    ) -> anyhow::Result<Self> {
        let docker = Docker::connect_with_local_defaults()?;
        let network_mode = detect_daemon_network(&docker).await;
        Ok(Self {
            docker,
            sessions,
            lifecycle,
            network_mode,
        })
    }

    pub async fn run_job(
        &self,
        payload: &WorkerJobPayload,
        config: &DaemonConfig,
    ) -> anyhow::Result<WorkerResult> {
        tokio::fs::create_dir_all(&payload.workspace_dir).await?;
        tokio::fs::create_dir_all(config.runtime_paths().job_logs_dir(payload.job_id)).await?;
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
                        network_mode: self.network_mode.clone(),
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
            .start_container(
                &container_id,
                None::<bollard::query_parameters::StartContainerOptions>,
            )
            .await
            .with_context(|| format!("failed to start worker container {}", container_id))?;
        self.lifecycle
            .mark_running(payload.job_id, &container_id)
            .await?;

        let mut wait = self.docker.wait_container(
            &container_id,
            None::<bollard::query_parameters::WaitContainerOptions>,
        );
        while let Some(next) = wait.next().await {
            next?;
        };
        let result = self
            .sessions
            .wait_for_result(
                payload.job_id,
                Duration::from_secs(config.worker_result_timeout_seconds),
            )
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "worker {} exited without uploading a result",
                    payload.job_id
                )
            })?;

        self.sessions.remove_session(payload.job_id);

        Ok(result)
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        for session in self.sessions.active_container_sessions().await {
            self.docker
                .kill_container(
                    &session.container_id,
                    None::<bollard::query_parameters::KillContainerOptions>,
                )
                .await
                .map_err(|error| {
                    warn!(
                        "failed to kill worker container {} during shutdown: {}",
                        session.container_id, error
                    );
                    error
                })
                .ok();
            self.sessions.remove_session(session.job_id);
        }
        Ok(())
    }
}

fn worker_env(payload: &WorkerJobPayload, config: &DaemonConfig, worker_id: &str) -> Vec<String> {
    vec![
        format!("SYNFORGE_WORKER_ID={worker_id}"),
        format!("SYNFORGE_WORKER_CONNECT_ADDR={}:8090", daemon_hostname()),
        format!(
            "SYNFORGE_WORKER_SOCKET_TIMEOUT_SECONDS={}",
            config.worker_socket_timeout_seconds
        ),
        format!("SYNFORGE_JOB_ID={}", payload.job_id),
    ]
}

fn daemon_hostname() -> String {
    std::env::var("HOSTNAME")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "daemon".to_string())
}

async fn detect_daemon_network(docker: &Docker) -> Option<String> {
    let hostname = std::env::var("HOSTNAME").ok()?;
    let inspect = docker
        .inspect_container(
            &hostname,
            None::<bollard::query_parameters::InspectContainerOptions>,
        )
        .await
        .ok()?;
    let networks = inspect.network_settings?.networks?;
    networks
        .keys()
        .find(|name| {
            name.as_str() != "bridge" && name.as_str() != "host" && name.as_str() != "none"
        })
        .cloned()
        .or_else(|| networks.keys().next().cloned())
}
