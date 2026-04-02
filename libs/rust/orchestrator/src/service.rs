use std::sync::Arc;
use std::path::PathBuf;

use anyhow::Context;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{CreateContainerOptionsBuilder, LogsOptionsBuilder, RemoveContainerOptionsBuilder};
use bollard::Docker;
use futures_util::StreamExt;
use synforge_core::{
    parse_mock_chroot, BrowseRepositoryRequest, BrowseRepositoryResponse, BuildJobResponse,
    CreatePackageRequest, DaemonConfig, EffectiveConfigDto, EffectiveConfigView, LogChunkResponse,
    MockChrootListResponse, PackageBuildHistoryResponse, PackageBuildInventoryEntry,
    PackageRepoFilesResponse, PackageResponse, RefreshRequest, RebuildRequest,
    RepoInventoryResponse, SynforgeError, UpdateRuntimeSettingsRequest,
};
use tokio::sync::{mpsc, Semaphore};
use tracing::{error, warn};
use uuid::Uuid;

use crate::db::{DieselStore, JobStore};
use crate::job_lifecycle::JobLifecycle;
use crate::packages::PackageSyncStore;
use crate::repo_manager::{FileRepoManager, RepoManager};
use crate::registry::PackageRegistry;
use crate::runner::BuildRunner;
use crate::scheduler::BuildScheduler;
use crate::sessions::WorkerSessionBroker;
use crate::workers::{DockerWorkerLauncher, WorkerLauncher};
use crate::worker_socket::start_worker_listener;

const POLLER_TICK_SECONDS: u64 = 30;

pub struct SynforgeService {
    config: DaemonConfig,
    store: DieselStore,
    registry: PackageRegistry,
    scheduler: BuildScheduler,
    runner: BuildRunner,
    lifecycle: Arc<JobLifecycle>,
    sessions: WorkerSessionBroker,
    worker_launcher: Arc<dyn WorkerLauncher>,
    queue_tx: mpsc::Sender<crate::scheduler::QueuedBuild>,
}

impl SynforgeService {
    async fn resolve_job_log_path(&self, job_id: Uuid) -> anyhow::Result<std::path::PathBuf> {
        let paths = self.config.runtime_paths();
        if let Some(path) = self.store.get_job_log_path(job_id).await? {
            return Ok(path);
        }

        let live_path = paths.job_worker_log_path(job_id);
        if tokio::fs::try_exists(&live_path).await? {
            return Ok(live_path);
        }

        Err(anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))
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

        Ok(path)
    }

    pub async fn new(config: DaemonConfig) -> anyhow::Result<Arc<Self>> {
        let paths = config.runtime_paths();
        let store = DieselStore::new(paths.database_path()).await?;
        let sessions = WorkerSessionBroker::new(paths.jobs_root().to_path_buf());
        let repo_manager = Arc::new(FileRepoManager);
        let lifecycle = Arc::new(JobLifecycle::new(config.clone(), store.clone(), repo_manager.clone()));
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
        worker_launcher: Arc<dyn WorkerLauncher>,
        repo_manager: Arc<dyn RepoManager>,
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

        let (queue_tx, queue_rx) = mpsc::channel(128);
        let service = Arc::new(Self {
            config,
            store: store.clone(),
            registry,
            scheduler,
            runner,
            lifecycle,
            sessions,
            worker_launcher,
            queue_tx,
        });
        start_worker_listener(service.config.worker_listen_addr.clone(), service.sessions.clone());
        service.start_queue_runner(queue_rx);
        service.start_poller();
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

    pub async fn list_packages(&self) -> anyhow::Result<Vec<PackageResponse>> {
        self.registry.list_packages().await
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
        let mut published_files_by_job = std::collections::HashMap::<Uuid, Vec<synforge_core::PublishedRepoFile>>::new();
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

    pub async fn get_repo_inventory(&self) -> anyhow::Result<RepoInventoryResponse> {
        Ok(RepoInventoryResponse {
            repo_files: self.store.list_published_repo_files().await?,
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
        docker
            .remove_container(
                &container_id,
                Some(RemoveContainerOptionsBuilder::default().force(true).build()),
            )
            .await
            .ok();

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
        request: synforge_core::UpdatePackageRequest,
    ) -> anyhow::Result<PackageResponse> {
        self.registry.update_package(package_name, request).await
    }

    pub async fn delete_package(&self, package_name: &str) -> anyhow::Result<()> {
        let jobs = self.store.list_jobs_for_package(package_name).await?;
        if jobs.iter().any(|entry| {
            matches!(
                entry.job.status,
                synforge_core::BuildStatus::Pending | synforge_core::BuildStatus::Running
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
            .enqueue_package(package_name, synforge_core::BuildTrigger::ManualRefresh, false, &self.queue_tx)
            .await
    }

    pub async fn trigger_rebuild(
        &self,
        package_name: &str,
        _request: RebuildRequest,
    ) -> anyhow::Result<BuildJobResponse> {
        self.scheduler
            .enqueue_package(package_name, synforge_core::BuildTrigger::ManualRebuild, true, &self.queue_tx)
            .await
    }

    pub async fn list_jobs(&self) -> anyhow::Result<Vec<BuildJobResponse>> {
        self.store.list_jobs().await
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
        cursor: Option<u64>,
        limit: Option<usize>,
    ) -> anyhow::Result<LogChunkResponse> {
        let path = self.resolve_job_log_path(job_id).await?;
        let bytes = tokio::fs::read(&path)
            .await
            .with_context(|| format!("failed to read {}", path.display()))?;
        let start = cursor.unwrap_or(0).min(bytes.len() as u64) as usize;
        let max_len = limit.unwrap_or(64 * 1024).clamp(1024, 512 * 1024);
        let end = (start + max_len).min(bytes.len());
        let contents = String::from_utf8_lossy(&bytes[start..end]).to_string();
        let cursor = end as u64;
        let complete = end >= bytes.len();
        Ok(LogChunkResponse {
            job_id,
            contents,
            cursor,
            complete,
        })
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

    pub async fn poll_once(&self) -> anyhow::Result<()> {
        self.scheduler.poll_once(&self.queue_tx).await
    }

    pub async fn graceful_shutdown(&self) {
        warn!("shutdown requested; stopping active worker containers");
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
    }

    fn start_queue_runner(self: &Arc<Self>, mut queue_rx: mpsc::Receiver<crate::scheduler::QueuedBuild>) {
        let runner = self.runner.clone();
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_builds));
        tokio::spawn(async move {
            while let Some(build) = queue_rx.recv().await {
                let runner = runner.clone();
                let semaphore = Arc::clone(&semaphore);
                match semaphore.acquire_owned().await {
                    Ok(permit) => {
                        tokio::spawn(async move {
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

    fn start_poller(self: &Arc<Self>) {
        let service = Arc::clone(self);
        tokio::spawn(async move {
            let mut ticker =
                tokio::time::interval(std::time::Duration::from_secs(POLLER_TICK_SECONDS));
            loop {
                ticker.tick().await;
                if let Err(error) = service.poll_once().await {
                    warn!("polling failed: {}", error);
                }
            }
        });
    }
}
