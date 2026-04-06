use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use bollard::Docker;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::CreateContainerOptionsBuilder;
use futures_util::StreamExt;
use synforge_core::{
    config::DaemonConfig,
    model::{WorkerAction, WorkerJobPayload, WorkerResult},
};
use tracing::{info, warn};

use crate::git_cache::{GitMirrorCache, WORKER_MIRROR_MOUNT_PATH};
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
        let (action, package_name, mock_chroot) = job_descriptor(payload);
        info!(
            job_id = %payload.job_id,
            action,
            package_name,
            mock_chroot,
            timeout_seconds = payload.timeout_seconds,
            "launching worker job"
        );
        tokio::fs::create_dir_all(&payload.workspace_dir).await?;
        tokio::fs::create_dir_all(config.runtime_paths().job_logs_dir(payload.job_id)).await?;
        tokio::fs::create_dir_all(config.runtime_paths().git_mirror_root()).await?;
        let session = self
            .sessions
            .create_session(payload.job_id, payload.clone())
            .await?;
        info!(
            job_id = %payload.job_id,
            worker_id = %session.worker_id,
            "worker session created"
        );

        let container_name = format!("synforge-worker-{}", payload.job_id);
        let repo_url = payload_repo_url(payload).map(str::to_string);
        let mirror_key = repo_url.as_deref().map(GitMirrorCache::mirror_key);
        let mirror_bind = format!(
            "{}:{}:ro",
            config.runtime_paths().git_mirror_root().display(),
            WORKER_MIRROR_MOUNT_PATH
        );
        info!(
            job_id = %payload.job_id,
            container_name = %container_name,
            worker_image = %config.worker_image,
            network_mode = ?self.network_mode,
            "creating worker container"
        );
        let container_with_bind = self
            .docker
            .create_container(
                Some(
                    CreateContainerOptionsBuilder::default()
                        .name(container_name.as_str())
                        .build(),
                ),
                worker_container_body(
                    config,
                    payload,
                    &session.worker_id,
                    self.network_mode.clone(),
                    Some(vec![mirror_bind.clone()]),
                    mirror_key.clone(),
                ),
            )
            .await;
        let container = match container_with_bind {
            Ok(container) => container,
            Err(error) => {
                warn!(
                    job_id = %payload.job_id,
                    container_name = %container_name,
                    bind = %mirror_bind,
                    error = %error,
                    "failed to create worker container with git mirror bind; retrying without mirror cache mount"
                );
                self.docker
                    .create_container(
                        Some(
                            CreateContainerOptionsBuilder::default()
                                .name(container_name.as_str())
                                .build(),
                        ),
                        worker_container_body(
                            config,
                            payload,
                            &session.worker_id,
                            self.network_mode.clone(),
                            None,
                            None,
                        ),
                    )
                    .await
                    .with_context(|| {
                        format!("failed to create worker container {}", container_name)
                    })?
            }
        };

        let container_id = container.id;
        info!(
            job_id = %payload.job_id,
            container_id = %container_id,
            "worker container created"
        );
        self.sessions
            .set_container_id(payload.job_id, container_id.clone())
            .await;
        info!(
            job_id = %payload.job_id,
            container_id = %container_id,
            "starting worker container"
        );
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
        info!(
            job_id = %payload.job_id,
            container_id = %container_id,
            "worker container started"
        );

        info!(
            job_id = %payload.job_id,
            container_id = %container_id,
            "waiting for worker container exit"
        );
        let mut wait = self.docker.wait_container(
            &container_id,
            None::<bollard::query_parameters::WaitContainerOptions>,
        );
        while let Some(next) = wait.next().await {
            next?;
        }
        info!(
            job_id = %payload.job_id,
            container_id = %container_id,
            "worker container exited; waiting for uploaded result"
        );
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

        match &result {
            WorkerResult::Parse(_) => {
                info!(job_id = %payload.job_id, "worker parse job completed")
            }
            WorkerResult::Build(build_result) => info!(
                job_id = %payload.job_id,
                status = ?build_result.status,
                artifact_count = build_result.artifacts.len(),
                "worker build job completed"
            ),
        }
        self.sessions.remove_session(payload.job_id);
        info!(
            job_id = %payload.job_id,
            "worker session removed after completion"
        );

        Ok(result)
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        let sessions = self.sessions.active_container_sessions().await;
        if !sessions.is_empty() {
            info!(
                active_container_count = sessions.len(),
                "stopping active worker containers during shutdown"
            );
        }
        for session in sessions {
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

fn worker_container_body(
    config: &DaemonConfig,
    payload: &WorkerJobPayload,
    worker_id: &str,
    network_mode: Option<String>,
    binds: Option<Vec<String>>,
    mirror_key: Option<String>,
) -> ContainerCreateBody {
    let mut env = worker_env(payload, config, worker_id);
    if let Some(mirror_key) = mirror_key {
        env.push(format!(
            "SYNFORGE_GIT_MIRROR_ROOT={WORKER_MIRROR_MOUNT_PATH}"
        ));
        env.push(format!("SYNFORGE_GIT_MIRROR_KEY={mirror_key}"));
    }
    ContainerCreateBody {
        image: Some(config.worker_image.clone()),
        env: Some(env),
        host_config: Some(HostConfig {
            auto_remove: Some(true),
            privileged: Some(true),
            extra_hosts: Some(vec!["host.docker.internal:host-gateway".to_string()]),
            network_mode,
            binds,
            ..Default::default()
        }),
        working_dir: Some("/synforge".to_string()),
        ..Default::default()
    }
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

fn job_descriptor(payload: &WorkerJobPayload) -> (&'static str, &str, &str) {
    match &payload.action {
        WorkerAction::Parse(parse) => ("parse", parse.package_name.as_str(), "-"),
        WorkerAction::Build(build) => (
            "build",
            build.package_name.as_str(),
            build.mock_chroot.as_str(),
        ),
    }
}

fn payload_repo_url(payload: &WorkerJobPayload) -> Option<&str> {
    match &payload.action {
        WorkerAction::Parse(parse) => Some(parse.source.repo_url.as_str()),
        WorkerAction::Build(build) => Some(build.package.source.repo_url.as_str()),
    }
}
