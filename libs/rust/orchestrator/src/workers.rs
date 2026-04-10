use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use bollard::Docker;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::CreateContainerOptionsBuilder;
use futures_util::StreamExt;
use synforge_core::{
    config::DaemonConfig,
    constants::DEFAULT_DAEMON_WORKER_SOCKET_PORT,
    model::{WorkerAction, WorkerJobPayload, WorkerResult},
};
use tracing::{info, warn};
use uuid::Uuid;

use crate::job_lifecycle::JobLifecycle;
use crate::sessions::WorkerSessionBroker;

#[derive(Debug, Clone, Copy, Default)]
struct WorkerResourceLimits {
    cpu_limit_millicores: Option<u64>,
    memory_limit_mb: Option<u64>,
    nano_cpus: Option<i64>,
    memory_bytes: Option<i64>,
}

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
        let runtime_paths = config.runtime_paths();
        tokio::fs::create_dir_all(runtime_paths.job_logs_dir(payload.job_id)).await?;
        let binds = Self::mock_mount_binds(payload, config).await?;
        let resource_limits = worker_resource_limits(payload)?;
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
        let worker_jobs_root = config.worker_jobs_root();
        info!(
            job_id = %payload.job_id,
            container_name = %container_name,
            worker_image = %config.worker_image,
            network_mode = ?self.network_mode,
            worker_jobs_root = ?worker_jobs_root,
            cpu_limit_millicores = ?resource_limits.cpu_limit_millicores,
            memory_limit_mb = ?resource_limits.memory_limit_mb,
            "creating worker container"
        );
        let container = self
            .docker
            .create_container(
                Some(
                    CreateContainerOptionsBuilder::default()
                        .name(container_name.as_str())
                        .build(),
                ),
                worker_container_body(
                    config,
                    &session.worker_id,
                    self.network_mode.clone(),
                    binds,
                    resource_limits,
                ),
            )
            .await
            .with_context(|| format!("failed to create worker container {}", container_name))?;

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
            if let Err(error) = next {
                warn!(
                    job_id = %payload.job_id,
                    container_id = %container_id,
                    error = %error,
                    "worker container wait stream returned error; continuing to uploaded result check"
                );
                break;
            }
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
        // NOTE: Session is NOT removed here - caller must call cleanup_session()
        // AFTER finalization to avoid cleanup race conditions.

        Ok(result)
    }

    /// Removes the session for a completed job. Call this AFTER finalization
    /// to avoid race conditions with the runtime cleanup worker.
    pub fn cleanup_session(&self, job_id: Uuid) {
        self.sessions.remove_session(job_id);
        info!(job_id = %job_id, "worker session cleaned up after finalization");
    }

    pub async fn kill_job(
        &self,
        job_id: Uuid,
        container_id: Option<String>,
        reason: &str,
    ) -> anyhow::Result<()> {
        let session_marked_failed = self.sessions.fail_build_result(job_id, reason).await?;
        let resolved_container_id = match container_id {
            Some(value) => Some(value),
            None => self.sessions.container_id_for_job(job_id).await,
        };
        let Some(container_id) = resolved_container_id else {
            if session_marked_failed {
                info!(
                    job_id = %job_id,
                    "kill requested; no container id available, but worker session was marked failed"
                );
            } else {
                warn!(
                    job_id = %job_id,
                    "kill requested for active job without container/session state"
                );
            }
            return Ok(());
        };

        info!(
            job_id = %job_id,
            container_id = %container_id,
            "killing worker container for job"
        );
        match self
            .docker
            .kill_container(
                &container_id,
                None::<bollard::query_parameters::KillContainerOptions>,
            )
            .await
        {
            Ok(()) => {
                info!(
                    job_id = %job_id,
                    container_id = %container_id,
                    "worker container killed"
                );
                Ok(())
            }
            Err(bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            }) => {
                warn!(
                    job_id = %job_id,
                    container_id = %container_id,
                    "worker container already exited before kill request"
                );
                Ok(())
            }
            Err(error) => Err(anyhow::anyhow!(
                "failed to kill worker container {} for {}: {}",
                container_id,
                job_id,
                error
            )),
        }
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        let sessions = self.sessions.active_container_sessions().await;
        if !sessions.is_empty() {
            info!(
                active_container_count = sessions.len(),
                "stopping active worker containers during shutdown"
            );
        }
        let mut first_error: Option<anyhow::Error> = None;
        for session in sessions {
            match self
                .docker
                .kill_container(
                    &session.container_id,
                    None::<bollard::query_parameters::KillContainerOptions>,
                )
                .await
            {
                Ok(()) => self.sessions.remove_session(session.job_id),
                Err(error) => {
                    warn!(
                        container_id = %session.container_id,
                        error = %error,
                        "failed to kill worker container during shutdown"
                    );
                    if first_error.is_none() {
                        first_error = Some(anyhow::anyhow!(
                            "failed to kill worker container {} during shutdown: {}",
                            session.container_id,
                            error
                        ));
                    }
                }
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
        Ok(())
    }

    async fn mock_mount_binds(
        payload: &WorkerJobPayload,
        config: &DaemonConfig,
    ) -> anyhow::Result<Option<Vec<String>>> {
        let WorkerAction::Build(build) = &payload.action else {
            return Ok(None);
        };

        let Some(host_jobs_root) = config.worker_jobs_host_path() else {
            warn!(
                job_id = %payload.job_id,
                package_name = %build.package.name,
                mock_chroot = %build.mock_chroot,
                ccache_enabled = build.package.ccache_enabled,
                "SYNFORGE_WORKER_JOBS_PATH not set; workers will run without dedicated mock or ccache bind mounts"
            );
            return Ok(None);
        };

        let host_mock_root = host_jobs_root.join(payload.job_id.to_string()).join("mock");
        let host_mock_lib = host_mock_root.join("lib");
        let host_mock_cache = host_mock_root.join("cache");

        // Create directories in container's view of the path
        let container_jobs_root = config.worker_jobs_root();
        let container_mock_root = container_jobs_root
            .join(payload.job_id.to_string())
            .join("mock");
        let container_mock_lib_dir = container_mock_root.join("lib");
        let container_mock_cache_dir = container_mock_root.join("cache");
        tokio::fs::create_dir_all(&container_mock_lib_dir).await?;
        tokio::fs::create_dir_all(&container_mock_cache_dir).await?;

        // Worker container mount targets
        let worker_mock_root = payload.workspace_dir.join("mock");
        let worker_mock_lib = worker_mock_root.join("lib");
        let worker_mock_cache = worker_mock_root.join("cache");

        let mut binds = vec![
            format!(
                "{}:{}:rw,z",
                host_mock_lib.display(),
                worker_mock_lib.display()
            ),
            format!(
                "{}:{}:rw,z",
                host_mock_cache.display(),
                worker_mock_cache.display()
            ),
        ];

        if build.package.ccache_enabled {
            let host_ccache_dir = host_jobs_root
                .join("ccache")
                .join(&build.package.name)
                .join(&build.mock_chroot);
            let container_ccache_dir = config
                .worker_jobs_root()
                .join("ccache")
                .join(&build.package.name)
                .join(&build.mock_chroot);
            tokio::fs::create_dir_all(&container_ccache_dir).await?;
            let worker_ccache_dir = payload.workspace_dir.join("ccache");
            binds.push(format!(
                "{}:{}:rw,z",
                host_ccache_dir.display(),
                worker_ccache_dir.display()
            ));
        }

        Ok(Some(binds))
    }
}

fn worker_container_body(
    config: &DaemonConfig,
    worker_id: &str,
    network_mode: Option<String>,
    binds: Option<Vec<String>>,
    resource_limits: WorkerResourceLimits,
) -> ContainerCreateBody {
    let cmd = vec![
        "--worker-id".to_string(),
        worker_id.to_string(),
        "--connect-addr".to_string(),
        format!("daemon:{}", DEFAULT_DAEMON_WORKER_SOCKET_PORT),
        "--socket-timeout-seconds".to_string(),
        config.worker_socket_timeout_seconds.to_string(),
    ];
    ContainerCreateBody {
        image: Some(config.worker_image.clone()),
        cmd: Some(cmd),
        host_config: Some(HostConfig {
            auto_remove: Some(true),
            privileged: Some(true),
            extra_hosts: Some(vec!["host.docker.internal:host-gateway".to_string()]),
            network_mode,
            binds,
            nano_cpus: resource_limits.nano_cpus,
            memory: resource_limits.memory_bytes,
            ..Default::default()
        }),
        working_dir: Some("/synforge".to_string()),
        ..Default::default()
    }
}

fn worker_resource_limits(payload: &WorkerJobPayload) -> anyhow::Result<WorkerResourceLimits> {
    let WorkerAction::Build(build) = &payload.action else {
        return Ok(WorkerResourceLimits::default());
    };
    let cpu_limit_millicores = build
        .package
        .cpu_limit_millicores
        .filter(|value| *value > 0);
    let memory_limit_mb = build.package.memory_limit_mb.filter(|value| *value > 0);
    let nano_cpus = cpu_limit_millicores
        .map(|value| {
            value
                .checked_mul(1_000_000)
                .ok_or_else(|| anyhow::anyhow!("cpu limit is too large: {}", value))
        })
        .transpose()?
        .map(i64::try_from)
        .transpose()
        .context("cpu limit exceeds i64 range")?;
    let memory_bytes = memory_limit_mb
        .map(|value| {
            value
                .checked_mul(1024 * 1024)
                .ok_or_else(|| anyhow::anyhow!("memory limit is too large: {}", value))
        })
        .transpose()?
        .map(i64::try_from)
        .transpose()
        .context("memory limit exceeds i64 range")?;
    Ok(WorkerResourceLimits {
        cpu_limit_millicores,
        memory_limit_mb,
        nano_cpus,
        memory_bytes,
    })
}

async fn detect_daemon_network(docker: &Docker) -> Option<String> {
    let hostname = daemon_container_hostname()?;
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

fn daemon_container_hostname() -> Option<String> {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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
