use std::time::Duration;

use anyhow::Context;
use futures_util::StreamExt;
use synforge_core::{
    config::DaemonConfig,
    constants::DEFAULT_WORKER_WAIT_GRACE_SECONDS,
    model::{WorkerJobPayload, WorkerResult},
};
use tracing::{info, warn};
use uuid::Uuid;

use super::DockerWorkerLauncher;
use crate::{
    container::{WORKER_CONTAINER_NAME_PREFIX, worker_container_body, worker_container_name},
    resources::{job_descriptor, worker_resource_limits},
};

impl DockerWorkerLauncher {
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

        let container_name = worker_container_name(payload.job_id);
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
                    bollard::query_parameters::CreateContainerOptionsBuilder::default()
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
            .mark_job_running(payload.job_id, &container_id)
            .await?;
        info!(
            job_id = %payload.job_id,
            container_id = %container_id,
            "worker container started"
        );

        // Bound the wait strictly longer than the worker's own build
        // timeout so a self-timing-out worker reports back first; past the
        // bound we assume the worker hung or never connected and force-kill
        // it. Without this the queue runner's concurrency permit (held for
        // the duration of run_job) would be pinned forever and the
        // container leaked.
        let wait_timeout = Duration::from_secs(
            payload
                .timeout_seconds
                .saturating_add(DEFAULT_WORKER_WAIT_GRACE_SECONDS),
        );
        info!(
            job_id = %payload.job_id,
            container_id = %container_id,
            wait_timeout_seconds = wait_timeout.as_secs(),
            "waiting for worker container exit"
        );
        let wait_outcome = tokio::time::timeout(wait_timeout, async {
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
        })
        .await;
        if wait_outcome.is_err() {
            warn!(
                job_id = %payload.job_id,
                container_id = %container_id,
                wait_timeout_seconds = wait_timeout.as_secs(),
                "worker container exceeded wait timeout; force-removing and failing job"
            );
            self.force_remove_container(&container_id).await;
            anyhow::bail!(
                "worker {} exceeded wait timeout of {}s",
                payload.job_id,
                wait_timeout.as_secs()
            );
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

        Ok(result)
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

    /// Best-effort force-remove of a single container by id. A 404 (already
    /// gone, e.g. via `auto_remove`) is treated as success; other errors are
    /// logged but not propagated, since callers use this on a path that is
    /// already failing the job.
    async fn force_remove_container(&self, container_id: &str) {
        match self
            .docker
            .remove_container(
                container_id,
                Some(
                    bollard::query_parameters::RemoveContainerOptionsBuilder::default()
                        .force(true)
                        .build(),
                ),
            )
            .await
        {
            Ok(()) => {}
            Err(bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            }) => {}
            Err(error) => {
                warn!(
                    container_id = %container_id,
                    error = %error,
                    "failed to force-remove worker container"
                );
            }
        }
    }

    /// Reap orphaned worker containers left behind by a daemon crash.
    /// Container names are deterministic (`synforge-worker-{job_id}`), so a
    /// retried job collides on `create_container` (name in use) unless these
    /// are removed first. Enumerates by the shared name prefix and
    /// force-removes each. Best-effort: individual removals that fail are
    /// logged and skipped so one stuck container can't block startup.
    pub async fn reap_orphan_worker_containers(&self) -> anyhow::Result<()> {
        let mut filters = std::collections::HashMap::new();
        filters.insert("name", vec![WORKER_CONTAINER_NAME_PREFIX]);
        let containers = self
            .docker
            .list_containers(Some(
                bollard::query_parameters::ListContainersOptionsBuilder::default()
                    .all(true)
                    .filters(&filters)
                    .build(),
            ))
            .await
            .context("failed to list worker containers for orphan reap")?;

        let mut removed = 0_u64;
        for container in containers {
            // The name filter is a substring match; require a real
            // `/synforge-worker-` segment so we never touch unrelated
            // containers that merely contain the substring.
            let is_worker = container.names.as_ref().is_some_and(|names| {
                names.iter().any(|name| {
                    name.trim_start_matches('/')
                        .starts_with(WORKER_CONTAINER_NAME_PREFIX)
                })
            });
            if !is_worker {
                continue;
            }
            let Some(container_id) = container.id else {
                continue;
            };
            info!(
                container_id = %container_id,
                container_names = ?container.names,
                "removing orphaned worker container at startup"
            );
            self.force_remove_container(&container_id).await;
            removed += 1;
        }
        if removed > 0 {
            info!(removed, "removed orphaned worker containers");
        }
        Ok(())
    }
}
