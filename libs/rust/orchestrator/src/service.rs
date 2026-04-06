use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use crate::db::{DieselStore, JobStore, UserStore};
use crate::job_lifecycle::JobLifecycle;
use crate::packages::PackageSyncStore;
use crate::registry::PackageRegistry;
use crate::repo_manager::FileRepoManager;
use crate::repo_signing::RepoSigningManager;
use crate::runner::BuildRunner;
use crate::scheduler::BuildScheduler;
use crate::service::config::apply_config_settings;
use crate::sessions::WorkerSessionBroker;
use crate::sync_tracker::SyncStatusTracker;
use crate::worker_socket::start_worker_listener;
use crate::workers::DockerWorkerLauncher;
use serde_json::Value;
use synforge_core::{
    api::{MockChrootListResponse, PageInfo, RepoSigningReconcileProgressView},
    config::DaemonConfig,
    model::UserPermission,
};
use tokio::sync::{Mutex, Semaphore, mpsc, watch};
use tokio_util::task::TaskTracker;
use tracing::{error, info, warn};
const DEFAULT_PAGE_SIZE: usize = 50;
const MAX_PAGE_SIZE: usize = 200;
const WORKER_LISTEN_ADDR: &str = "0.0.0.0:8090";

mod cache;
mod config;
mod jobs;
mod logs;
mod packages;
mod repo;
mod signing;
mod sync;
mod users;

pub struct SynforgeService {
    config: DaemonConfig,
    store: DieselStore,
    registry: PackageRegistry,
    scheduler: BuildScheduler,
    runner: BuildRunner,
    lifecycle: Arc<JobLifecycle>,
    sessions: WorkerSessionBroker,
    worker_launcher: Arc<DockerWorkerLauncher>,
    task_tracker: TaskTracker,
    queue_tx: mpsc::Sender<crate::scheduler::QueuedBuild>,
    shutdown_tx: watch::Sender<bool>,
    mock_chroot_cache: Arc<Mutex<MockChrootCacheState>>,
    signing_reconcile_progress: Arc<Mutex<Option<RepoSigningReconcileProgressView>>>,
}

#[derive(Debug, Clone)]
struct MockChrootCacheEntry {
    worker_image: String,
    fetched_at: Instant,
    fetched_at_unix_seconds: i64,
    response: MockChrootListResponse,
}

#[derive(Debug, Default)]
struct MockChrootCacheState {
    entry: Option<MockChrootCacheEntry>,
    hit_count: u64,
    miss_count: u64,
    stale_served_count: u64,
}

impl SynforgeService {
    pub async fn health_check(&self) -> anyhow::Result<()> {
        self.store.health_check().await?;

        let paths = self.config.runtime_paths();
        for path in [paths.packages_dir(), paths.repo_dir(), paths.jobs_root()] {
            if !tokio::fs::try_exists(path).await? {
                anyhow::bail!("required runtime path is missing: {}", path.display());
            }
        }

        Ok(())
    }

    pub async fn new(mut config: DaemonConfig) -> anyhow::Result<Arc<Self>> {
        info!("initializing synforge service");
        let store = DieselStore::new(&config.database_url, config.db_pool_size).await?;
        apply_startup_runtime_overrides(&store, &mut config).await?;
        let paths = config.runtime_paths();
        let sessions = WorkerSessionBroker::new(paths.jobs_root().to_path_buf());
        let repo_manager = Arc::new(FileRepoManager);
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
        )
        .await
    }

    pub async fn new_with_components(
        config: DaemonConfig,
        store: DieselStore,
        worker_launcher: Arc<DockerWorkerLauncher>,
        repo_manager: Arc<FileRepoManager>,
        sessions: WorkerSessionBroker,
        lifecycle: Arc<JobLifecycle>,
    ) -> anyhow::Result<Arc<Self>> {
        config
            .validate()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        info!(
            packages_dir = %config.runtime_paths().packages_dir().display(),
            repo_dir = %config.runtime_paths().repo_dir().display(),
            jobs_root = %config.runtime_paths().jobs_root().display(),
            max_concurrent_builds = config.max_concurrent_builds,
            queue_buffer_size = config.queue_buffer_size,
            poller_tick_seconds = config.poller_tick_seconds,
            "configuring synforge runtime"
        );
        let paths = config.runtime_paths();
        tokio::fs::create_dir_all(paths.packages_dir()).await?;
        tokio::fs::create_dir_all(paths.repo_dir()).await?;
        tokio::fs::create_dir_all(paths.jobs_root()).await?;
        repo_manager.ensure_repo(&config).await?;
        store
            .abort_unfinished_jobs("daemon restarted before job completed")
            .await?;

        let package_store = PackageSyncStore::new(
            paths.packages_dir().to_path_buf(),
            config.clone(),
            worker_launcher.clone(),
            store.clone(),
        );
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
            mock_chroot_cache: Arc::new(Mutex::new(MockChrootCacheState::default())),
            signing_reconcile_progress: Arc::new(Mutex::new(None)),
        });
        start_worker_listener(
            WORKER_LISTEN_ADDR.to_string(),
            service.store.clone(),
            service.sessions.clone(),
            service.task_tracker.clone(),
            shutdown_rx.clone(),
        );
        info!(
            worker_listener = WORKER_LISTEN_ADDR,
            "worker socket listener started"
        );
        service.start_queue_runner(queue_rx, shutdown_rx.clone());
        service.start_poller(shutdown_rx);
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
        mut queue_rx: mpsc::Receiver<crate::scheduler::QueuedBuild>,
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

async fn apply_startup_runtime_overrides(
    store: &DieselStore,
    config: &mut DaemonConfig,
) -> anyhow::Result<()> {
    let settings = store.list_runtime_settings().await?;
    apply_config_settings(config, &settings, true)?;
    let mut updates = BTreeMap::new();
    if !settings.contains_key("session_secret") {
        updates.insert(
            "session_secret".to_string(),
            Value::String(config.session_secret.clone()),
        );
    }
    if !config.bootstrap_completed && store.user_count().await? > 0 {
        config.bootstrap_completed = true;
        updates.insert("bootstrap_completed".to_string(), Value::Bool(true));
    } else if config.bootstrap_completed && !settings.contains_key("bootstrap_completed") {
        updates.insert("bootstrap_completed".to_string(), Value::Bool(true));
    }
    if !updates.is_empty() {
        store.upsert_runtime_settings(updates).await?;
    }
    config
        .validate()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    if !config.signing_enabled {
        return Ok(());
    }

    let signing_manager = RepoSigningManager;
    let status = signing_manager.status(config).await?;
    if status.key_present {
        return Ok(());
    }

    let Some(armored_private_key) = settings
        .get("signing_private_key_armored")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };

    signing_manager.remove_all_keys(config).await?;
    let imported = signing_manager
        .import_private_key(config, armored_private_key)
        .await?;
    if config.signing_key_id.as_deref() != Some(imported.key_id.as_str()) {
        config.signing_key_id = Some(imported.key_id.clone());
        let mut updates = BTreeMap::new();
        updates.insert("signing_key_id".to_string(), Value::String(imported.key_id));
        store.upsert_runtime_settings(updates).await?;
    }

    Ok(())
}

fn normalize_pagination(limit: Option<usize>, offset: Option<usize>) -> (usize, usize) {
    (
        limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE),
        offset.unwrap_or(0),
    )
}

fn build_page_info(limit: usize, offset: usize, total: u64, returned: usize) -> PageInfo {
    PageInfo {
        limit,
        offset,
        returned,
        total: Some(total),
        has_more: (offset as u64) + (returned as u64) < total,
    }
}

fn validate_user_handle(handle: &str) -> anyhow::Result<()> {
    if handle.is_empty() {
        anyhow::bail!("user handle must not be empty");
    }
    if !handle
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        anyhow::bail!("user handle may only contain letters, digits, '.', '_' and '-'");
    }
    Ok(())
}

fn validate_display_name(display_name: &str) -> anyhow::Result<()> {
    if display_name.trim().is_empty() {
        anyhow::bail!("display_name must not be empty");
    }
    Ok(())
}

fn validate_password(password: &str) -> anyhow::Result<()> {
    if password.is_empty() {
        anyhow::bail!("password must not be empty");
    }
    Ok(())
}

fn validate_permissions(permissions: &[UserPermission]) -> anyhow::Result<()> {
    if permissions.is_empty() {
        anyhow::bail!("at least one permission is required");
    }
    Ok(())
}
