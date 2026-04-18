use std::time::Duration;

use anyhow::Context;
use futures_util::StreamExt;
use synforge_core::{
    config::DaemonConfig,
    model::{WorkerJobPayload, WorkerResult},
};
use tracing::{info, warn};
use uuid::Uuid;

use super::DockerWorkerLauncher;
use crate::{
    container::worker_container_body,
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
}
