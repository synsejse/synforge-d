use std::sync::Arc;

use super::apply_startup_runtime_overrides;
use super::runtime_cache::RuntimeCache;
use synforge_core::{
    api::{
        MockChrootListResponse, RefreshAllPackagesProgressView, RepoSigningReconcileProgressView,
    },
    config::DaemonConfig,
    constants::DEFAULT_DAEMON_WORKER_LISTEN_ADDR,
};
use synforge_runtime::{
    BuildRunner, BuildScheduler, DockerWorkerLauncher, FileRepoManager, JobLifecycle,
    JobObjectStorage, PackageRegistry, PackageSyncStore, QueuedBuild, SyncStatusTracker,
    WorkerSessionBroker, start_worker_listener,
};
use synforge_store::{DieselStore, JobStore};
use tokio::sync::{Mutex, Semaphore, mpsc, watch};
use tokio_util::task::TaskTracker;
use tracing::{error, info, warn};

pub struct SynforgeService {
    pub(super) config: DaemonConfig,
    pub(super) store: DieselStore,
    pub(super) registry: PackageRegistry,
    pub(super) scheduler: BuildScheduler,
    pub(super) runner: BuildRunner,
    pub(super) lifecycle: Arc<JobLifecycle>,
    pub(super) sessions: WorkerSessionBroker,
    pub(super) worker_launcher: Arc<DockerWorkerLauncher>,
    pub(super) task_tracker: TaskTracker,
    pub(super) queue_tx: mpsc::Sender<QueuedBuild>,
    pub(super) shutdown_tx: watch::Sender<bool>,
    pub(super) runtime_cache: RuntimeCache,
    pub(super) object_storage: JobObjectStorage,
    pub(super) mock_chroot_cache: Arc<Mutex<MockChrootCacheState>>,
    pub(super) refresh_all_packages_progress: Arc<Mutex<Option<RefreshAllPackagesProgressView>>>,
    pub(super) signing_reconcile_progress: Arc<Mutex<Option<RepoSigningReconcileProgressView>>>,
}

#[derive(Debug, Clone)]
pub(super) struct MockChrootCacheEntry {
    pub(super) worker_image: String,
    pub(super) fetched_at_unix_seconds: i64,
    pub(super) response: MockChrootListResponse,
}

#[derive(Debug, Default)]
pub(super) struct MockChrootCacheState {
    pub(super) entry: Option<MockChrootCacheEntry>,
    pub(super) hit_count: u64,
    pub(super) miss_count: u64,
    pub(super) stale_served_count: u64,
}

impl SynforgeService {
    pub async fn health_check(&self) -> anyhow::Result<()> {
        self.store.health_check().await?;
        self.runtime_cache.health_check().await?;
        self.object_storage.health_check().await?;

        let paths = self.config.runtime_paths();
        for path in [
            paths.repo_dir(),
            paths.jobs_root(),
            paths.cache_root(),
            paths.work_root(),
        ] {
            if !tokio::fs::try_exists(path).await? {
                anyhow::bail!("required runtime path is missing: {}", path.display());
            }
        }
        if !tokio::fs::try_exists(paths.signing_root()).await? {
            anyhow::bail!(
                "required runtime path is missing: {}",
                paths.signing_root().display()
            );
        }

        Ok(())
    }

    pub async fn new(mut config: DaemonConfig) -> anyhow::Result<Arc<Self>> {
        info!("initializing synforge service");
        let store = DieselStore::new(&config.database_url, config.db_pool_size).await?;
        apply_startup_runtime_overrides(&store, &mut config).await?;
        let runtime_cache = RuntimeCache::new(&config).await?;
        let object_storage = JobObjectStorage::from_config(&config).await?;
        let paths = config.runtime_paths();
        let sessions =
            WorkerSessionBroker::new(paths.jobs_root().to_path_buf(), object_storage.clone());
        let repo_manager = Arc::new(FileRepoManager::new(object_storage.clone()));
        let lifecycle = Arc::new(JobLifecycle::new(
            config.clone(),
            store.clone(),
            repo_manager.clone(),
        ));
        let worker_launcher =
            Arc::new(DockerWorkerLauncher::new(sessions.clone(), lifecycle.clone()).await?);
        Self::new_with_components(
            config,
            store,
            worker_launcher,
            repo_manager,
            sessions,
            lifecycle,
            runtime_cache,
            object_storage,
        )
        .await
    }

    pub(crate) async fn new_with_components(
        config: DaemonConfig,
        store: DieselStore,
        worker_launcher: Arc<DockerWorkerLauncher>,
        repo_manager: Arc<FileRepoManager>,
        sessions: WorkerSessionBroker,
        lifecycle: Arc<JobLifecycle>,
        runtime_cache: RuntimeCache,
        object_storage: JobObjectStorage,
    ) -> anyhow::Result<Arc<Self>> {
        config
            .validate()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        info!(
            repo_dir = %config.runtime_paths().repo_dir().display(),
            jobs_root = %config.runtime_paths().jobs_root().display(),
            cache_root = %config.runtime_paths().cache_root().display(),
            work_root = %config.runtime_paths().work_root().display(),
            signing_root = %config.runtime_paths().signing_root().display(),
            worker_jobs_root = %config.worker_jobs_root().display(),
            worker_ccache_root = %config.worker_ccache_root().display(),
            max_concurrent_builds = config.max_concurrent_builds,
            queue_buffer_size = config.queue_buffer_size,
            poller_tick_seconds = config.poller_tick_seconds,
            "configuring synforge runtime"
        );
        let paths = config.runtime_paths();
        tokio::fs::create_dir_all(paths.repo_dir()).await?;
        tokio::fs::create_dir_all(paths.jobs_root()).await?;
        tokio::fs::create_dir_all(paths.cache_root()).await?;
        tokio::fs::create_dir_all(paths.work_root()).await?;
        tokio::fs::create_dir_all(paths.signing_root()).await?;
        repo_manager.ensure_repo(&config).await?;
        store
            .abort_unfinished_jobs("daemon restarted before job completed")
            .await?;

        let package_store =
            PackageSyncStore::new(config.clone(), worker_launcher.clone(), store.clone());
        if let Err(error) = package_store.cleanup_git_mirror_cache().await {
            warn!(error = %error, "failed to cleanup git mirror cache at startup");
        }
        let sync_tracker = Some(SyncStatusTracker::new(store.clone()));
        let registry = PackageRegistry::new(store.clone(), package_store, sync_tracker);
        let scheduler = BuildScheduler::new(store.clone(), registry.clone());
        let runner = BuildRunner::new(config.clone(), worker_launcher.clone(), lifecycle.clone());

        let (queue_tx, queue_rx) = mpsc::channel(config.queue_buffer_size);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let task_tracker = TaskTracker::new();
        let service = Arc::new(Self {
            config,
            store: store.clone(),
            registry,
            scheduler,
            runner,
            lifecycle,
            sessions,
            worker_launcher,
            task_tracker,
            queue_tx,
            shutdown_tx,
            runtime_cache,
            object_storage,
            mock_chroot_cache: Arc::new(Mutex::new(MockChrootCacheState::default())),
            refresh_all_packages_progress: Arc::new(Mutex::new(None)),
            signing_reconcile_progress: Arc::new(Mutex::new(None)),
        });
        start_worker_listener(
            DEFAULT_DAEMON_WORKER_LISTEN_ADDR.to_string(),
            service.store.clone(),
            service.sessions.clone(),
            service.task_tracker.clone(),
            shutdown_rx.clone(),
        );
        info!(
            worker_listener = DEFAULT_DAEMON_WORKER_LISTEN_ADDR,
            "worker socket listener started"
        );
        service.start_queue_runner(queue_rx, shutdown_rx.clone());
        service.start_poller(shutdown_rx.clone());
        service.start_job_usage_sampler(shutdown_rx.clone());
        service.start_runtime_cleanup_worker(shutdown_rx);
        info!("synforge service background workers started");
        Ok(service)
    }

    pub fn config(&self) -> &DaemonConfig {
        &self.config
    }

    pub async fn poll_once(&self) -> anyhow::Result<()> {
        self.scheduler.poll_once(&self.queue_tx).await
    }

    pub async fn graceful_shutdown(&self) {
        warn!("shutdown requested; stopping active worker containers");
        let _ = self.shutdown_tx.send(true);
        self.task_tracker.close();
        if let Err(error) = self.worker_launcher.shutdown().await {
            error!("failed to stop active worker containers: {}", error);
        }
        if let Err(error) = self
            .lifecycle
            .abort_unfinished_jobs("daemon shutdown requested")
            .await
        {
            error!("failed to abort unfinished jobs during shutdown: {}", error);
        }
        self.task_tracker.wait().await;
    }

    fn start_queue_runner(
        self: &Arc<Self>,
        mut queue_rx: mpsc::Receiver<QueuedBuild>,
        mut shutdown_rx: watch::Receiver<bool>,
    ) {
        let runner = self.runner.clone();
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_builds));
        let task_tracker = self.task_tracker.clone();
        info!(
            max_concurrent_builds = self.config.max_concurrent_builds,
            "starting build queue runner"
        );
        task_tracker.clone().spawn(async move {
            loop {
                let maybe_build = tokio::select! {
                    _ = shutdown_rx.changed() => {
                        break;
                    }
                    build = queue_rx.recv() => build,
                };
                let Some(build) = maybe_build else {
                    break;
                };
                info!(
                    job_id = %build.job_id,
                    package_name = %build.package.name,
                    mock_chroot = %build.mock_chroot,
                    "build dequeued for execution"
                );
                let runner = runner.clone();
                let semaphore = Arc::clone(&semaphore);
                let task_tracker = task_tracker.clone();
                match semaphore.acquire_owned().await {
                    Ok(permit) => {
                        task_tracker.spawn(async move {
                            let _permit = permit;
                            if let Err(error) = runner.process_build(build).await {
                                error!("build processing failed: {}", error);
                            }
                        });
                    }
                    Err(error) => error!("failed to acquire build semaphore: {}", error),
                }
            }
        });
    }

    fn start_poller(self: &Arc<Self>, mut shutdown_rx: watch::Receiver<bool>) {
        let service = Arc::clone(self);
        info!(
            poller_tick_seconds = service.config.poller_tick_seconds,
            "starting poller"
        );
        self.task_tracker.spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(
                service.config.poller_tick_seconds,
            ));
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        break;
                    }
                    _ = ticker.tick() => {
                        if let Err(error) = service.poll_once().await {
                            warn!("polling failed: {}", error);
                        }
                    }
                }
            }
        });
    }
}
