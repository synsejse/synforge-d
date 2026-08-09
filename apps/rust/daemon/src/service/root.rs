#[path = "root/background.rs"]
mod background;
#[path = "root/startup.rs"]
mod startup;

use std::sync::Arc;
use std::time::Instant;

use synforge_core::config::DaemonConfig;
use synforge_database::DieselStore;
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
        self.build_service.poll_once(&self.package_deps()).await
    }
}
