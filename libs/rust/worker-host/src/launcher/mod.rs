mod binds;
mod queue;
mod runtime;

use std::sync::Arc;

use async_trait::async_trait;
use bollard::Docker;
use tracing::info;
use uuid::Uuid;

use crate::{WorkerSessionBroker, container::detect_daemon_network};

#[async_trait]
pub trait WorkerJobTracker: Send + Sync {
    async fn mark_job_running(&self, job_id: Uuid, container_id: &str) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct DockerWorkerLauncher {
    docker: Docker,
    sessions: WorkerSessionBroker,
    lifecycle: Arc<dyn WorkerJobTracker>,
    network_mode: Option<String>,
}

impl DockerWorkerLauncher {
    pub async fn new(
        sessions: WorkerSessionBroker,
        lifecycle: Arc<dyn WorkerJobTracker>,
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

    pub fn cleanup_session(&self, job_id: Uuid) {
        self.sessions.remove_session(job_id);
        info!(job_id = %job_id, "worker session cleaned up after finalization");
    }
}
