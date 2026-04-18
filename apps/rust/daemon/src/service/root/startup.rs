use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{mpsc, watch};
use tokio_util::task::TaskTracker;
use tracing::{info, warn};

use crate::service::{SynforgeService, apply_startup_runtime_overrides};
use synforge_core::{
    config::DaemonConfig,
    constants::DEFAULT_DAEMON_WORKER_LISTEN_ADDR,
    model::{WorkerJobPayload, WorkerResult},
};
use synforge_database::DieselStore;
use synforge_git_sync::{
    PackageSyncStore, RuntimeGitRegistryAdapter, SyncStatusTracker, WorkerParseRunner,
};
use synforge_publish::{FileRepoManager, JobObjectStorage, RepoService, WorkerOutputStorage};
use synforge_state::{
    MockChrootCache, RefreshAllPackagesProgressState, RuntimeCache, SigningReconcileProgressState,
};
use synforge_worker_host::{
    BuildRunner, BuildService, DockerWorkerLauncher, JobLifecycle, WorkerSessionBroker,
    start_worker_listener,
};

#[derive(Clone)]
struct DaemonWorkerParseRunner {
    launcher: Arc<DockerWorkerLauncher>,
}

pub(crate) struct SynforgeStartupComponents {
    config: DaemonConfig,
    store: DieselStore,
    worker_launcher: Arc<DockerWorkerLauncher>,
    repo_manager: Arc<FileRepoManager>,
    sessions: WorkerSessionBroker,
    lifecycle: Arc<JobLifecycle>,
    runtime_cache: RuntimeCache,
    object_storage: JobObjectStorage,
}

#[async_trait]
impl WorkerParseRunner for DaemonWorkerParseRunner {
    async fn run_job(
        &self,
        payload: &WorkerJobPayload,
        config: &DaemonConfig,
    ) -> anyhow::Result<WorkerResult> {
        self.launcher.run_job(payload, config).await
    }
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
        let sessions = WorkerSessionBroker::new(
            paths.jobs_root().to_path_buf(),
            Arc::new(object_storage.clone()) as Arc<dyn WorkerOutputStorage>,
        );
        let repo_manager = Arc::new(FileRepoManager::new(object_storage.clone()));
        let lifecycle = Arc::new(JobLifecycle::new(
            config.clone(),
            store.clone(),
            repo_manager.clone(),
        ));
        let worker_launcher =
            Arc::new(DockerWorkerLauncher::new(sessions.clone(), lifecycle.clone()).await?);
        Self::new_with_components(SynforgeStartupComponents {
            config,
            store,
            worker_launcher,
            repo_manager,
            sessions,
            lifecycle,
            runtime_cache,
            object_storage,
        })
        .await
    }

    pub(crate) async fn new_with_components(
        components: SynforgeStartupComponents,
    ) -> anyhow::Result<Arc<Self>> {
        let SynforgeStartupComponents {
            config,
            store,
            worker_launcher,
            repo_manager,
            sessions,
            lifecycle,
            runtime_cache,
            object_storage,
        } = components;
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
        lifecycle
            .abort_unfinished_jobs("daemon restarted before job completed")
            .await?;

        let package_store = PackageSyncStore::new(
            config.clone(),
            Arc::new(DaemonWorkerParseRunner {
                launcher: worker_launcher.clone(),
            }),
            store.clone(),
        );
        if let Err(error) = package_store.cleanup_git_mirror_cache().await {
            warn!(error = %error, "failed to cleanup git mirror cache at startup");
        }
        let sync_tracker = Some(SyncStatusTracker::new(store.clone()));
        let registry = RuntimeGitRegistryAdapter::new(store.clone(), package_store, sync_tracker);
        let runner = BuildRunner::new(config.clone(), worker_launcher.clone(), lifecycle.clone());
        let build_service = BuildService::default();
        let repo_service = RepoService;

        let (queue_tx, queue_rx) = mpsc::channel(config.queue_buffer_size);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let task_tracker = TaskTracker::new();
        let service = Arc::new(Self {
            config,
            store: store.clone(),
            registry,
            build_service,
            repo_service,
            runner,
            lifecycle,
            sessions,
            worker_launcher,
            task_tracker,
            queue_tx,
            shutdown_tx,
            runtime_cache,
            object_storage,
            mock_chroot_cache: MockChrootCache::default(),
            refresh_all_packages_progress: RefreshAllPackagesProgressState::default(),
            signing_reconcile_progress: SigningReconcileProgressState::default(),
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
}
