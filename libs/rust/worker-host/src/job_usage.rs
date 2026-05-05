use std::collections::HashSet;
use std::time::Duration;

use bollard::Docker;
use bollard::query_parameters::StatsOptionsBuilder;
use futures_util::StreamExt;
use synforge_core::{
    api::{JobResourceUsageListResponse, JobResourceUsageResponse, JobResourceUsageSample},
    model::{format_timestamp, now_utc},
};
use synforge_state::RuntimeCache;
use tokio::sync::watch;
use tokio_util::task::TaskTracker;
use tracing::warn;
use uuid::Uuid;

use crate::{ActiveWorkerSession, WorkerSessionBroker};

#[derive(Clone)]
pub struct JobUsageService {
    sessions: WorkerSessionBroker,
    runtime_cache: RuntimeCache,
}

impl JobUsageService {
    pub fn new(sessions: WorkerSessionBroker, runtime_cache: RuntimeCache) -> Self {
        Self {
            sessions,
            runtime_cache,
        }
    }

    pub async fn list_job_resource_usage(&self) -> anyhow::Result<JobResourceUsageListResponse> {
        let mut samples = self.runtime_cache.list_job_usage_samples().await?;
        samples.sort_by_key(|sample| sample.job_id);
        Ok(JobResourceUsageListResponse { samples })
    }

    pub async fn get_job_resource_usage(
        &self,
        job_id: Uuid,
    ) -> anyhow::Result<JobResourceUsageResponse> {
        let sample = self.runtime_cache.get_job_usage_sample(job_id).await?;
        Ok(JobResourceUsageResponse { sample })
    }

    pub fn start_sampler(&self, task_tracker: TaskTracker, mut shutdown_rx: watch::Receiver<bool>) {
        let service = self.clone();
        task_tracker.spawn(async move {
            let mut backoff_seconds: u64 = 1;
            loop {
                if *shutdown_rx.borrow() {
                    break;
                }
                let docker = match Docker::connect_with_local_defaults() {
                    Ok(docker) => {
                        backoff_seconds = 1;
                        docker
                    }
                    Err(error) => {
                        warn!(
                            error = %error,
                            backoff_seconds,
                            "failed to initialize docker client for job usage polling; will retry"
                        );
                        let sleep = tokio::time::sleep(Duration::from_secs(backoff_seconds));
                        tokio::pin!(sleep);
                        tokio::select! {
                            _ = shutdown_rx.changed() => break,
                            _ = &mut sleep => {}
                        }
                        backoff_seconds = (backoff_seconds * 2).min(60);
                        continue;
                    }
                };
                let mut ticker = tokio::time::interval(Duration::from_secs(1));
                ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                let mut consecutive_failures: u32 = 0;
                loop {
                    tokio::select! {
                        _ = shutdown_rx.changed() => return,
                        _ = ticker.tick() => {
                            match service.poll_once(&docker).await {
                                Ok(()) => consecutive_failures = 0,
                                Err(error) => {
                                    consecutive_failures += 1;
                                    warn!(
                                        error = %error,
                                        consecutive_failures,
                                        "job usage polling cycle failed"
                                    );
                                    if consecutive_failures >= 5 {
                                        warn!(
                                            "reconnecting docker client after repeated job usage polling failures"
                                        );
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    async fn poll_once(&self, docker: &Docker) -> anyhow::Result<()> {
        let active_sessions = self.sessions.active_container_sessions().await;
        let active_ids = active_sessions
            .iter()
            .map(|session| session.job_id)
            .collect::<HashSet<_>>();
        for sample in self.runtime_cache.list_job_usage_samples().await? {
            if !active_ids.contains(&sample.job_id) {
                self.runtime_cache
                    .remove_job_usage_sample(sample.job_id)
                    .await?;
            }
        }

        for session in active_sessions {
            match sample_active_session_usage(docker, &session).await {
                Ok(Some(sample)) => {
                    self.runtime_cache.set_job_usage_sample(&sample).await?;
                }
                Ok(None) => {
                    self.runtime_cache
                        .remove_job_usage_sample(session.job_id)
                        .await?;
                }
                Err(error) => {
                    warn!(
                        job_id = %session.job_id,
                        container_id = %session.container_id,
                        error = %error,
                        "failed to read live container usage"
                    );
                }
            }
        }
        Ok(())
    }
}

async fn sample_active_session_usage(
    docker: &Docker,
    session: &ActiveWorkerSession,
) -> anyhow::Result<Option<JobResourceUsageSample>> {
    let mut stream = docker.stats(
        &session.container_id,
        Some(
            StatsOptionsBuilder::new()
                .stream(false)
                .one_shot(true)
                .build(),
        ),
    );
    let Some(next) = stream.next().await else {
        return Ok(None);
    };
    let stats = match next {
        Ok(stats) => stats,
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => return Ok(None),
        Err(error) => {
            return Err(anyhow::anyhow!(
                "failed to read stats for container {}: {}",
                session.container_id,
                error
            ));
        }
    };
    let memory_usage_bytes = stats
        .memory_stats
        .as_ref()
        .and_then(|memory| memory.usage)
        .unwrap_or(0);
    let memory_limit_bytes = stats
        .memory_stats
        .as_ref()
        .and_then(|memory| memory.limit)
        .unwrap_or(0);
    Ok(Some(JobResourceUsageSample {
        job_id: session.job_id,
        container_id: session.container_id.clone(),
        memory_usage_bytes,
        memory_limit_bytes,
        collected_at: format_timestamp(now_utc()),
    }))
}
