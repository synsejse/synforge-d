use std::collections::{HashMap, HashSet};
use std::time::Duration;

use bollard::Docker;
use bollard::query_parameters::StatsOptionsBuilder;
use futures_util::StreamExt;
use synforge_core::{
    api::{JobResourceUsageResponse, JobResourceUsageSample},
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

    pub async fn list_job_resource_usage(&self) -> anyhow::Result<Vec<JobResourceUsageSample>> {
        let mut samples = self.runtime_cache.list_job_usage_samples().await?;
        samples.sort_by_key(|sample| sample.job_id);
        Ok(samples)
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
            let docker = match Docker::connect_with_local_defaults() {
                Ok(docker) => docker,
                Err(error) => {
                    warn!(error = %error, "failed to initialize docker client for job usage polling");
                    return;
                }
            };
            let mut ticker = tokio::time::interval(Duration::from_secs(1));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            // Previous CPU counters per container, kept across ticks so we can
            // derive a CPU% from the delta (one-shot stats carry no precpu).
            let mut cpu_history: HashMap<String, CpuSnapshot> = HashMap::new();
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => break,
                    _ = ticker.tick() => {
                        if let Err(error) = service.poll_once(&docker, &mut cpu_history).await {
                            warn!(error = %error, "job usage polling cycle failed");
                        }
                    }
                }
            }
        });
    }

    async fn poll_once(
        &self,
        docker: &Docker,
        cpu_history: &mut HashMap<String, CpuSnapshot>,
    ) -> anyhow::Result<()> {
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

        // Drop CPU history for containers no longer active so the map can't
        // grow without bound.
        let active_container_ids = active_sessions
            .iter()
            .map(|session| session.container_id.clone())
            .collect::<HashSet<_>>();
        cpu_history.retain(|container_id, _| active_container_ids.contains(container_id));

        for session in active_sessions {
            match sample_active_session_usage(docker, &session, cpu_history).await {
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

#[derive(Clone, Copy)]
pub(crate) struct CpuSnapshot {
    total_usage: u64,
    system_usage: u64,
}

async fn sample_active_session_usage(
    docker: &Docker,
    session: &ActiveWorkerSession,
    cpu_history: &mut HashMap<String, CpuSnapshot>,
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

    // CPU% from the delta between this and the previous sample. One-shot
    // stats carry no precpu, so we keep our own previous counters.
    let cpu_stats = stats.cpu_stats.as_ref();
    let cpu_total = cpu_stats
        .and_then(|cpu| cpu.cpu_usage.as_ref())
        .and_then(|usage| usage.total_usage)
        .unwrap_or(0);
    let system_usage = cpu_stats.and_then(|cpu| cpu.system_cpu_usage).unwrap_or(0);
    let online_cpus = cpu_stats
        .and_then(|cpu| cpu.online_cpus)
        .or_else(|| {
            cpu_stats
                .and_then(|cpu| cpu.cpu_usage.as_ref())
                .and_then(|usage| usage.percpu_usage.as_ref())
                .map(|cores| cores.len() as u32)
        })
        .unwrap_or(1)
        .max(1);
    let cpu_percent = match cpu_history.get(&session.container_id) {
        Some(prev) if system_usage > prev.system_usage && cpu_total >= prev.total_usage => {
            let cpu_delta = (cpu_total - prev.total_usage) as f64;
            let system_delta = (system_usage - prev.system_usage) as f64;
            ((cpu_delta / system_delta) * f64::from(online_cpus) * 100.0).max(0.0)
        }
        _ => 0.0,
    };
    cpu_history.insert(
        session.container_id.clone(),
        CpuSnapshot {
            total_usage: cpu_total,
            system_usage,
        },
    );

    Ok(Some(JobResourceUsageSample {
        job_id: session.job_id,
        container_id: session.container_id.clone(),
        memory_usage_bytes,
        memory_limit_bytes,
        cpu_percent,
        online_cpus,
        collected_at: format_timestamp(now_utc()),
    }))
}
