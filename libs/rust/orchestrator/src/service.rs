use std::sync::Arc;
use std::path::PathBuf;

use anyhow::Context;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{CreateContainerOptionsBuilder, LogsOptionsBuilder};
use bollard::Docker;
use futures_util::StreamExt;
use synforge_core::{
    api::{
        BrowseRepositoryRequest, BrowseRepositoryResponse, BuildJobResponse, CreatePackageRequest,
        EffectiveConfigDto, EffectiveConfigView, LogChunkResponse, LogManifestResponse, LogMetaResponse,
        LogSource, LogSourceType, MockChrootListResponse, PackageBuildHistoryResponse,
        PackageBuildInventoryEntry, PackageRepoFilesResponse, PackageResponse, PruneJobsResponse,
        RefreshRequest, RebuildRequest, RepoInventoryResponse, UpdatePackageRequest,
        UpdateRuntimeSettingsRequest,
    },
    config::DaemonConfig,
    error::SynforgeError,
    model::{BuildStatus, BuildTrigger, PublishedRepoFile},
    package::parse_mock_chroot,
};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::{mpsc, watch, Semaphore};
use tokio_util::task::TaskTracker;
use tracing::{error, warn};
use uuid::Uuid;

use crate::db::{DieselStore, JobStore};
use crate::job_lifecycle::JobLifecycle;
use crate::packages::PackageSyncStore;
use crate::repo_manager::FileRepoManager;
use crate::registry::PackageRegistry;
use crate::runner::BuildRunner;
use crate::scheduler::BuildScheduler;
use crate::sessions::WorkerSessionBroker;
use crate::workers::DockerWorkerLauncher;
use crate::worker_socket::start_worker_listener;
const DEFAULT_PAGE_SIZE: usize = 50;
const MAX_PAGE_SIZE: usize = 200;

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

    async fn resolve_job_log_path(&self, job_id: Uuid, source: &str) -> anyhow::Result<std::path::PathBuf> {
        let paths = self.config.runtime_paths();
        let logs_dir = paths.job_logs_dir(job_id);
        let path = logs_dir.join(source);
        if tokio::fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(path);
        }
        Err(anyhow::anyhow!(SynforgeError::NotFound(format!(
            "log source {} for job {}",
            source, job_id
        ))))
    }

    pub async fn resolve_job_artifact_path(
        &self,
        job_id: Uuid,
        relative_repo_path: &str,
    ) -> anyhow::Result<PathBuf> {
        let job = self
            .store
            .get_job(job_id).await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))?;
        let artifact = job
            .artifacts
            .into_iter()
            .find(|artifact| artifact.relative_repo_path == PathBuf::from(relative_repo_path))
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(relative_repo_path.to_string())))?;

        let path = artifact.path;
        if !tokio::fs::try_exists(&path).await? {
            return Err(anyhow::anyhow!(SynforgeError::NotFound(path.display().to_string())));
        }

        let artifacts_root = tokio::fs::canonicalize(self.config.runtime_paths().job_artifacts_dir(job_id))
            .await
            .with_context(|| format!("failed to resolve job artifact root for {}", job_id))?;
        let resolved_path = tokio::fs::canonicalize(&path)
            .await
            .with_context(|| format!("failed to resolve artifact path {}", path.display()))?;
        if !resolved_path.starts_with(&artifacts_root) {
            anyhow::bail!(
                "resolved artifact path {} escapes job artifact root {}",
                resolved_path.display(),
                artifacts_root.display()
            );
        }

        Ok(resolved_path)
    }

    pub async fn new(config: DaemonConfig) -> anyhow::Result<Arc<Self>> {
        let paths = config.runtime_paths();
        let store = DieselStore::new(paths.database_path(), config.db_pool_size).await?;
        let sessions = WorkerSessionBroker::new(paths.jobs_root().to_path_buf());
        let repo_manager = Arc::new(FileRepoManager);
        let lifecycle = Arc::new(JobLifecycle::new(
            config.clone(),
            store.clone(),
            repo_manager.clone(),
        ));
        let worker_launcher = Arc::new(DockerWorkerLauncher::new(sessions.clone(), lifecycle.clone())?);
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
        let paths = config.runtime_paths();
        tokio::fs::create_dir_all(paths.packages_dir()).await?;
        tokio::fs::create_dir_all(paths.repo_dir()).await?;
        tokio::fs::create_dir_all(paths.jobs_root()).await?;
        tokio::fs::create_dir_all(paths.temp_root()).await?;
        repo_manager.ensure_repo(&config).await?;
        store
            .abort_unfinished_jobs("daemon restarted before job completed")
            .await?;

        let package_store = PackageSyncStore::new(
            paths.packages_dir().to_path_buf(),
            config.clone(),
            worker_launcher.clone(),
        );
        let registry = PackageRegistry::new(store.clone(), package_store);
        let scheduler = BuildScheduler::new(store.clone(), registry.clone());
        let runner = BuildRunner::new(
            config.clone(),
            worker_launcher.clone(),
            lifecycle.clone(),
            scheduler.clone(),
        );

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
        });
        start_worker_listener(
            service.config.worker_listen_addr.clone(),
            service.sessions.clone(),
            service.task_tracker.clone(),
            shutdown_rx.clone(),
        );
        service.start_queue_runner(queue_rx, shutdown_rx.clone());
        service.start_poller(shutdown_rx);
        Ok(service)
    }

    pub fn config(&self) -> &DaemonConfig {
        &self.config
    }

    pub async fn effective_config(&self) -> EffectiveConfigDto {
        let paths = self.config.runtime_paths();
        let public_base_url = self
            .store
            .get_public_base_url_override()
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| self.config.public_base_url.clone());
        EffectiveConfigDto {
            config: EffectiveConfigView {
                listen_addr: self.config.listen_addr.clone(),
                bearer_token: self.config.bearer_token.clone(),
                runtime_root: self.config.runtime_root.clone(),
                database_path: paths.database_path().to_path_buf(),
                packages_dir: paths.packages_dir().to_path_buf(),
                repo_dir: paths.repo_dir().to_path_buf(),
                jobs_root: paths.jobs_root().to_path_buf(),
                worker_image: self.config.worker_image.clone(),
                max_concurrent_builds: self.config.max_concurrent_builds,
                db_pool_size: self.config.db_pool_size,
                queue_buffer_size: self.config.queue_buffer_size,
                poller_tick_seconds: self.config.poller_tick_seconds,
                worker_result_timeout_seconds: self.config.worker_result_timeout_seconds,
                worker_socket_timeout_seconds: self.config.worker_socket_timeout_seconds,
                git_operation_timeout_seconds: self.config.git_operation_timeout_seconds,
                public_base_url,
                worker_listen_addr: self.config.worker_listen_addr.clone(),
                worker_connect_addr: self.config.worker_connect_addr.clone(),
            },
        }
    }

    pub async fn update_runtime_settings(
        &self,
        request: UpdateRuntimeSettingsRequest,
    ) -> anyhow::Result<EffectiveConfigDto> {
        if request.public_base_url.trim().is_empty() {
            anyhow::bail!("public_base_url must not be empty");
        }
        self.store
            .set_public_base_url_override(&request.public_base_url)
            .await?;
        Ok(self.effective_config().await)
    }

    pub async fn list_packages(&self, limit: Option<usize>, offset: Option<usize>) -> anyhow::Result<Vec<PackageResponse>> {
        let (limit, offset) = normalize_pagination(limit, offset);
        self.registry.list_packages(limit, offset).await
    }

    pub async fn get_package(&self, package_name: &str) -> anyhow::Result<PackageResponse> {
        self.registry.get_package(package_name).await
    }

    pub async fn get_package_build_history(
        &self,
        package_name: &str,
    ) -> anyhow::Result<PackageBuildHistoryResponse> {
        self.registry.get_package(package_name).await?;
        let jobs = self.store.list_jobs_for_package(package_name).await?;
        let published_files = self
            .store
            .list_published_repo_files_for_package(package_name)
            .await?;
        let mut published_files_by_job = std::collections::HashMap::<Uuid, Vec<PublishedRepoFile>>::new();
        for file in &published_files {
            published_files_by_job
                .entry(file.job_id)
                .or_default()
                .push(file.clone());
        }
        let builds = jobs
            .into_iter()
            .map(|build| PackageBuildInventoryEntry {
                repo_files: published_files_by_job.remove(&build.job.id).unwrap_or_default(),
                build,
            })
            .collect();
        Ok(PackageBuildHistoryResponse {
            package_name: package_name.to_string(),
            builds,
        })
    }

    pub async fn get_package_repo_files(
        &self,
        package_name: &str,
    ) -> anyhow::Result<PackageRepoFilesResponse> {
        self.registry.get_package(package_name).await?;
        Ok(PackageRepoFilesResponse {
            package_name: package_name.to_string(),
            repo_files: self
                .store
                .list_published_repo_files_for_package(package_name)
            .await?,
        })
    }

    pub async fn get_repo_inventory(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> anyhow::Result<RepoInventoryResponse> {
        let (limit, offset) = normalize_pagination(limit, offset);
        Ok(RepoInventoryResponse {
            repo_files: self.store.list_published_repo_files(limit, offset).await?,
        })
    }

    pub async fn create_package(
        &self,
        request: CreatePackageRequest,
    ) -> anyhow::Result<PackageResponse> {
        self.registry.create_package(request).await
    }

    pub async fn list_mock_chroots(&self) -> anyhow::Result<MockChrootListResponse> {
        let docker = Docker::connect_with_local_defaults()?;
        let container_name = format!("synforge-mock-chroots-{}", Uuid::now_v7());
        let container = docker
            .create_container(
                Some(
                    CreateContainerOptionsBuilder::default()
                        .name(container_name.as_str())
                        .build(),
                ),
                ContainerCreateBody {
                    image: Some(self.config.worker_image.clone()),
                    entrypoint: Some(vec!["bash".to_string(), "-lc".to_string()]),
                    cmd: Some(vec!["mock --list-chroots".to_string()]),
                    host_config: Some(HostConfig {
                        auto_remove: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .await?;

        let container_id = container.id;
        docker
            .start_container(&container_id, None::<bollard::query_parameters::StartContainerOptions>)
            .await?;

        let mut stdout = String::new();
        let mut logs = docker.logs(
            &container_id,
            Some(
                LogsOptionsBuilder::default()
                    .follow(true)
                    .stdout(true)
                    .stderr(false)
                    .timestamps(false)
                    .tail("all")
                    .build(),
            ),
        );
        while let Some(item) = logs.next().await {
            stdout.push_str(&String::from_utf8_lossy(item?.into_bytes().as_ref()));
        }

        let mut wait = docker
            .wait_container(&container_id, None::<bollard::query_parameters::WaitContainerOptions>);
        while let Some(next) = wait.next().await {
            next?;
        }
        let mut chroots = stdout
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .filter_map(|line| line.split_whitespace().next())
            .filter(|name| name.starts_with("fedora-"))
            .filter(|name| parse_mock_chroot(name).is_some())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        chroots.sort();
        chroots.dedup();
        Ok(MockChrootListResponse { chroots })
    }

    pub async fn browse_repository(
        &self,
        request: BrowseRepositoryRequest,
    ) -> anyhow::Result<BrowseRepositoryResponse> {
        self.registry.browse_repository(&request.repo_url).await
    }

    pub async fn update_package(
        &self,
        package_name: &str,
        request: UpdatePackageRequest,
    ) -> anyhow::Result<PackageResponse> {
        self.registry.update_package(package_name, request).await
    }

    pub async fn delete_package(&self, package_name: &str) -> anyhow::Result<()> {
        let jobs = self.store.list_jobs_for_package(package_name).await?;
        if jobs.iter().any(|entry| {
            matches!(
                entry.job.status,
                BuildStatus::Pending | BuildStatus::Running
            )
        }) {
            anyhow::bail!("cannot delete package {} while a job is pending or running", package_name);
        }
        for job in jobs {
            self.delete_job(job.job.id).await?;
        }
        self.registry.delete_package(package_name).await
    }

    pub async fn trigger_refresh(
        &self,
        package_name: &str,
        _request: RefreshRequest,
    ) -> anyhow::Result<BuildJobResponse> {
        self.scheduler
            .enqueue_package(package_name, BuildTrigger::ManualRefresh, false, &self.queue_tx)
            .await
    }

    pub async fn trigger_rebuild(
        &self,
        package_name: &str,
        _request: RebuildRequest,
    ) -> anyhow::Result<BuildJobResponse> {
        self.scheduler
            .enqueue_package(package_name, BuildTrigger::ManualRebuild, true, &self.queue_tx)
            .await
    }

    pub async fn list_jobs(&self, limit: Option<usize>, offset: Option<usize>) -> anyhow::Result<Vec<BuildJobResponse>> {
        let (limit, offset) = normalize_pagination(limit, offset);
        self.store.list_jobs(limit, offset).await
    }

    pub async fn get_job(&self, job_id: Uuid) -> anyhow::Result<BuildJobResponse> {
        self.store
            .get_job(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))
    }

    pub async fn get_job_log_chunk(
        &self,
        job_id: Uuid,
        source: Option<String>,
        cursor: Option<u64>,
        offset: Option<i64>,
        limit: Option<usize>,
    ) -> anyhow::Result<LogChunkResponse> {
        let source_name = source.unwrap_or_else(|| "worker.log".to_string());
        let path = self.resolve_job_log_path(job_id, &source_name).await?;
        
        let mut file = tokio::fs::File::open(&path)
            .await
            .with_context(|| format!("failed to open {}", path.display()))?;
        
        let file_size = file.metadata().await?.len();
        let max_len = limit.unwrap_or(64 * 1024).clamp(1024, 512 * 1024) as u64;
        let start = ((cursor.unwrap_or(0).min(file_size) as i128) + offset.unwrap_or(0) as i128)
            .clamp(0, file_size as i128) as u64;
        let read_len = max_len.min(file_size.saturating_sub(start));
        
        if read_len == 0 {
            return Ok(LogChunkResponse {
                job_id,
                source: source_name,
                contents: String::new(),
                start_line: count_lines_before(&path, start).await?,
                cursor: start,
                complete: true,
            });
        }
        
        file.seek(std::io::SeekFrom::Start(start)).await?;
        let mut buffer = vec![0u8; read_len as usize];
        let bytes_read = file.read(&mut buffer).await?;
        buffer.truncate(bytes_read);
        
        // Find UTF-8 safe boundary to avoid splitting multi-byte characters
        let safe_len = find_utf8_boundary(&buffer);
        buffer.truncate(safe_len);
        
        let contents = String::from_utf8_lossy(&buffer).into_owned();
        let start_line = count_lines_before(&path, start).await?;
        let new_cursor = start + safe_len as u64;
        let complete = new_cursor >= file_size;
        
        Ok(LogChunkResponse {
            job_id,
            source: source_name,
            contents,
            start_line,
            cursor: new_cursor,
            complete,
        })
    }

    pub async fn get_job_log_meta(
        &self,
        job_id: Uuid,
        source: Option<String>,
    ) -> anyhow::Result<LogMetaResponse> {
        let source_name = source.unwrap_or_else(|| "worker.log".to_string());
        let path = self.resolve_job_log_path(job_id, &source_name).await?;
        let file_size = tokio::fs::metadata(&path)
            .await
            .with_context(|| format!("failed to stat {}", path.display()))?
            .len();
        Ok(LogMetaResponse {
            job_id,
            source: source_name,
            file_size,
            max_cursor: file_size,
        })
    }
    
    pub async fn get_job_log_manifest(&self, job_id: Uuid) -> anyhow::Result<LogManifestResponse> {
        let paths = self.config.runtime_paths();
        let logs_dir = paths.job_logs_dir(job_id);
        
        let mut sources = Vec::new();
        
        for (name, filename, source_type) in [("Worker Output", "worker.log", LogSourceType::Raw)] {
            let log_path = logs_dir.join(filename);
            if tokio::fs::try_exists(&log_path).await.unwrap_or(false) {
                if let Ok(meta) = tokio::fs::metadata(&log_path).await {
                    sources.push(LogSource {
                        name: name.to_string(),
                        path: filename.to_string(),
                        size: meta.len(),
                        source_type,
                    });
                }
            }
        }

        for (name, filename) in [
            ("Mock Root Log", "mock-root.log"),
            ("Mock Build Log", "mock-build.log"),
            ("Mock State Log", "mock-state.log"),
        ] {
            let log_path = logs_dir.join(filename);
            if tokio::fs::try_exists(&log_path).await.unwrap_or(false) {
                if let Ok(meta) = tokio::fs::metadata(&log_path).await {
                    sources.push(LogSource {
                        name: name.to_string(),
                        path: filename.to_string(),
                        size: meta.len(),
                        source_type: LogSourceType::Raw,
                    });
                }
            }
        }

        Ok(LogManifestResponse { job_id, sources })
    }

    pub async fn delete_job(&self, job_id: Uuid) -> anyhow::Result<BuildJobResponse> {
        let published_files = self.store.list_published_repo_files_for_job(job_id).await?;
        self.lifecycle.remove_published_files(&published_files).await?;
        let deleted = self
            .store
            .delete_job(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))?;
        self.lifecycle.remove_job_runtime(job_id).await?;
        Ok(deleted)
    }

    pub async fn prune_failed_jobs(&self) -> anyhow::Result<PruneJobsResponse> {
        let jobs = self.store.list_jobs(10_000, 0).await?;
        let mut deleted_jobs = Vec::new();
        for job in jobs {
            if matches!(job.job.status, BuildStatus::Failed | BuildStatus::TimedOut) {
                deleted_jobs.push(self.delete_job(job.job.id).await?);
            }
        }
        Ok(PruneJobsResponse { deleted_jobs })
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
        self.task_tracker.spawn(async move {
            let mut ticker =
                tokio::time::interval(std::time::Duration::from_secs(service.config.poller_tick_seconds));
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

fn normalize_pagination(limit: Option<usize>, offset: Option<usize>) -> (usize, usize) {
    (
        limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE),
        offset.unwrap_or(0),
    )
}

/// Find a safe UTF-8 boundary in a byte slice.
/// Returns the largest index <= buffer.len() that doesn't split a UTF-8 character.
fn find_utf8_boundary(buffer: &[u8]) -> usize {
    if buffer.is_empty() {
        return 0;
    }
    
    // Start from the end and work backwards to find a valid boundary
    let mut end = buffer.len();
    
    // Check if we're in the middle of a UTF-8 sequence
    // UTF-8 continuation bytes have the pattern 10xxxxxx (0x80-0xBF)
    while end > 0 {
        let byte = buffer[end - 1];
        
        // If this byte is ASCII or a UTF-8 start byte, we're at a valid boundary
        if byte < 0x80 {
            // ASCII byte - valid boundary
            break;
        } else if byte >= 0xC0 {
            // UTF-8 start byte (110xxxxx, 1110xxxx, or 11110xxx)
            // Check if there's enough room for the full character
            let char_len = if byte >= 0xF0 {
                4
            } else if byte >= 0xE0 {
                3
            } else {
                2
            };
            
            if end + char_len - 1 <= buffer.len() {
                // Full character fits, include it
                break;
            } else {
                // Character is truncated, exclude it
                end -= 1;
            }
        } else {
            // Continuation byte (10xxxxxx) - go back to find start
            end -= 1;
        }
    }
    
    end
}

async fn count_lines_before(path: &std::path::Path, end_offset: u64) -> anyhow::Result<u64> {
    if end_offset == 0 {
        return Ok(1);
    }

    let mut file = tokio::fs::File::open(path)
        .await
        .with_context(|| format!("failed to open {}", path.display()))?;
    file.seek(std::io::SeekFrom::Start(0)).await?;

    let mut remaining = end_offset;
    let mut buffer = vec![0_u8; 64 * 1024];
    let mut line_count = 0_u64;

    while remaining > 0 {
        let read_len = remaining.min(buffer.len() as u64) as usize;
        let bytes_read = file.read(&mut buffer[..read_len]).await?;
        if bytes_read == 0 {
            break;
        }
        line_count += buffer[..bytes_read]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count() as u64;
        remaining = remaining.saturating_sub(bytes_read as u64);
    }

    Ok(line_count + 1)
}
