#[path = "root/background.rs"]
mod background;
#[path = "root/startup.rs"]
mod startup;

use std::time::Instant;
use std::{collections::HashMap, sync::Arc};

use synforge_core::config::DaemonConfig;
use synforge_core::sync::SyncTriggerType;
use synforge_database::{DieselStore, SyncStore};
use synforge_git_sync::RuntimeGitRegistryAdapter;
use synforge_state::{
    MockChrootCache, RefreshAllPackagesProgressState, RuntimeCache, SigningReconcileProgressState,
};
use synforge_worker_host::{
    BuildRunner, BuildService, DockerWorkerLauncher, JobLifecycle, LogBroadcaster, QueuedBuild,
    WorkerSessionBroker,
};
use tokio::sync::{Mutex, mpsc, watch};
use tokio_util::task::TaskTracker;
use tracing::{info, warn};

pub(super) type HealthProbeCache = Arc<Mutex<Option<(Instant, Result<(), String>)>>>;

pub struct SynforgeService {
    pub(super) config: DaemonConfig,
    pub(super) store: DieselStore,
    pub(super) registry: RuntimeGitRegistryAdapter,
    pub(super) build_service: BuildService,
    pub(super) runner: BuildRunner,
    pub(super) lifecycle: Arc<JobLifecycle>,
    pub(super) sessions: WorkerSessionBroker,
    pub(super) log_broadcaster: LogBroadcaster,
    pub(super) worker_launcher: Arc<DockerWorkerLauncher>,
    pub(super) task_tracker: TaskTracker,
    pub(super) queue_tx: mpsc::Sender<QueuedBuild>,
    pub(super) sync_queue_tx: mpsc::Sender<uuid::Uuid>,
    pub(super) shutdown_tx: watch::Sender<bool>,
    pub(super) runtime_cache: RuntimeCache,
    pub(super) mock_chroot_cache: MockChrootCache,
    pub(super) refresh_all_packages_progress: RefreshAllPackagesProgressState,
    pub(super) signing_reconcile_progress: SigningReconcileProgressState,
    pub(super) health_cache: HealthProbeCache,
}

impl SynforgeService {
    pub fn config(&self) -> &DaemonConfig {
        &self.config
    }

    pub async fn poll_once(&self) -> anyhow::Result<()> {
        let now = time::OffsetDateTime::now_utc();
        let last_sync: HashMap<String, time::OffsetDateTime> = self
            .store
            .last_sync_at_per_package()
            .await?
            .into_iter()
            .collect();
        for package in self.registry.list_definitions().await? {
            if !package.enabled || !package.source.poll {
                continue;
            }
            let interval = time::Duration::seconds(package.poll_interval_seconds.max(1) as i64);
            if last_sync
                .get(&package.name)
                .is_some_and(|last| *last + interval > now)
            {
                continue;
            }
            match self
                .enqueue_package_sync(&package.name, SyncTriggerType::Poll, None, None, None)
                .await
            {
                Ok(response) if response.created => {
                    info!(package_name = %package.name, "scheduled package source poll");
                }
                Ok(_) => {}
                Err(error) => {
                    warn!(package_name = %package.name, %error, "failed to schedule source poll");
                }
            }
        }
        Ok(())
    }
}
