use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use bollard::Docker;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{CreateContainerOptionsBuilder, LogsOptionsBuilder};
use futures_util::StreamExt;
use synforge_core::{
    api::{
        BrowseRepositoryRequest, BrowseRepositoryResponse, BuildJobListResponse, BuildJobResponse,
        ChangePasswordRequest, CreatePackageRequest, CreateUserRequest, JobArtifactListResponse,
        JobArtifactMetaResponse, MockChrootListResponse, PackageActionResponse,
        PackageActionTargetResult, PackageBuildHistoryResponse, PackageBuildInventoryEntry,
        PackageListResponse, PackageResponse, PageInfo, PruneJobsResponse, RebuildRequest,
        RefreshRequest, RepoInventoryResponse, RepoSummaryResponse, SessionResponse,
        UpdatePackageRequest, UpdateUserRequest, UserListResponse, UserMetricsResponse,
        UserResponse,
    },
    config::DaemonConfig,
    error::SynforgeError,
    model::{BuildStatus, BuildTrigger, PublishedRepoFile, UserAccount, UserPermission},
    package::parse_mock_chroot,
};
use tokio::sync::{Semaphore, mpsc, watch};
use tokio_util::task::TaskTracker;
use tracing::{error, warn};
use uuid::Uuid;

use crate::auth::{hash_password, verify_password};
use crate::db::{DieselStore, JobStore};
use crate::job_lifecycle::JobLifecycle;
use crate::packages::PackageSyncStore;
use crate::registry::PackageRegistry;
use crate::repo_manager::FileRepoManager;
use crate::runner::BuildRunner;
use crate::scheduler::BuildScheduler;
use crate::sessions::WorkerSessionBroker;
use crate::worker_socket::start_worker_listener;
use crate::workers::DockerWorkerLauncher;
const DEFAULT_PAGE_SIZE: usize = 50;
const MAX_PAGE_SIZE: usize = 200;
const WORKER_LISTEN_ADDR: &str = "0.0.0.0:8090";

mod config;
mod logs;

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

    pub async fn resolve_job_artifact_path(
        &self,
        job_id: Uuid,
        relative_artifact_path: &str,
    ) -> anyhow::Result<PathBuf> {
        let job = self
            .store
            .get_job(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))?;
        let artifact = job
            .artifacts
            .into_iter()
            .find(|artifact| artifact.file == std::path::Path::new(relative_artifact_path))
            .ok_or_else(|| {
                anyhow::anyhow!(SynforgeError::NotFound(relative_artifact_path.to_string()))
            })?;

        let path = self
            .config
            .runtime_paths()
            .job_artifacts_dir(job_id)
            .join(artifact.storage_path());
        if !tokio::fs::try_exists(&path).await? {
            return Err(anyhow::anyhow!(SynforgeError::NotFound(
                path.display().to_string()
            )));
        }

        let artifacts_root =
            tokio::fs::canonicalize(self.config.runtime_paths().job_artifacts_dir(job_id))
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
        let store = DieselStore::new(&config.database_url, config.db_pool_size).await?;
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
        });
        start_worker_listener(
            WORKER_LISTEN_ADDR.to_string(),
            service.store.clone(),
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

    pub async fn list_packages(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<PackageListResponse> {
        let (limit, offset) = normalize_pagination(limit, offset);
        let total = self.store.count_packages(search.clone(), enabled).await?;
        let packages = self
            .store
            .list_packages(limit, offset, search, enabled)
            .await?;
        Ok(PackageListResponse {
            page: build_page_info(limit, offset, total, packages.len()),
            packages,
        })
    }

    pub async fn get_package(&self, package_name: &str) -> anyhow::Result<PackageResponse> {
        self.registry.get_package(package_name).await
    }

    pub async fn get_package_build_history(
        &self,
        package_name: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> anyhow::Result<PackageBuildHistoryResponse> {
        self.registry.get_package(package_name).await?;
        let (limit, offset) = normalize_pagination(limit, offset);
        let total = self
            .store
            .count_jobs(None, Some(package_name.to_string()), None, false)
            .await?;
        let jobs = self
            .store
            .list_jobs(
                limit,
                offset,
                None,
                Some(package_name.to_string()),
                None,
                false,
            )
            .await?;
        let published_files = self
            .store
            .list_published_repo_files_for_package(package_name)
            .await?;
        let returned = jobs.len();
        let mut published_files_by_job =
            std::collections::HashMap::<Uuid, Vec<PublishedRepoFile>>::new();
        for file in &published_files {
            published_files_by_job
                .entry(file.job_id)
                .or_default()
                .push(file.clone());
        }
        let builds = jobs
            .into_iter()
            .map(|build| PackageBuildInventoryEntry {
                repo_files: published_files_by_job
                    .remove(&build.job.id)
                    .unwrap_or_default(),
                build,
            })
            .collect();
        Ok(PackageBuildHistoryResponse {
            package_name: package_name.to_string(),
            builds,
            page: build_page_info(limit, offset, total, returned),
        })
    }

    pub async fn get_repo_inventory(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<synforge_core::model::ArtifactKind>,
    ) -> anyhow::Result<RepoInventoryResponse> {
        let (limit, offset) = normalize_pagination(limit, offset);
        let total = self
            .store
            .count_published_repo_files(package_name.clone(), mock_chroot.clone(), kind)
            .await?;
        let repo_files = self
            .store
            .list_published_repo_files(limit, offset, package_name, mock_chroot, kind)
            .await?;
        Ok(RepoInventoryResponse {
            page: build_page_info(limit, offset, total, repo_files.len()),
            repo_files,
        })
    }

    pub async fn get_repo_summary(&self) -> anyhow::Result<RepoSummaryResponse> {
        let (package_count, target_count, build_count) =
            self.store.get_repo_distinct_counts().await?;
        Ok(RepoSummaryResponse {
            package_count,
            target_count,
            build_count,
            stored_bytes: self.store.sum_published_repo_file_bytes().await?,
            published_file_count: self
                .store
                .count_published_repo_files(None, None, None)
                .await?,
            targets: self.store.list_repo_target_summaries().await?,
            recent_files: self.store.list_recent_published_repo_files(10).await?,
        })
    }

    pub async fn authenticate_user(
        &self,
        handle: &str,
        password: &str,
        required: UserPermission,
    ) -> anyhow::Result<UserAccount> {
        let Some(record) = self.store.get_user_auth_by_handle(handle).await? else {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        };
        if !record.user.active {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        }
        if !verify_password(&record.password_hash, password)? {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        }
        if !record.user.has_permission(required) {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        }
        Ok(record.user)
    }

    pub async fn authorize_user(
        &self,
        user_id: Uuid,
        required: UserPermission,
    ) -> anyhow::Result<UserAccount> {
        let Some(summary) = self.store.get_user(user_id).await? else {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        };
        if !summary.user.active || !summary.user.has_permission(required) {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        }
        Ok(summary.user)
    }

    pub async fn get_session(&self, user: UserAccount) -> SessionResponse {
        SessionResponse { user }
    }

    pub async fn list_users(&self) -> anyhow::Result<UserListResponse> {
        let users = self
            .store
            .list_users()
            .await?
            .into_iter()
            .map(|summary| UserResponse {
                user: summary.user,
                metrics: summary.metrics,
            })
            .collect();
        Ok(UserListResponse { users })
    }

    pub async fn create_user(&self, request: CreateUserRequest) -> anyhow::Result<UserResponse> {
        validate_user_handle(&request.handle)?;
        validate_display_name(&request.display_name)?;
        validate_password(&request.password)?;
        validate_permissions(&request.permissions)?;
        if self
            .store
            .get_user_by_handle(&request.handle)
            .await?
            .is_some()
        {
            return Err(anyhow::anyhow!(SynforgeError::Conflict(format!(
                "user handle {} already exists",
                request.handle
            ))));
        }
        let password_hash = hash_password(&request.password)?;
        let summary = self
            .store
            .create_user(
                &request.handle,
                &request.display_name,
                &password_hash,
                request.active,
                &request.permissions,
            )
            .await?;
        Ok(UserResponse {
            user: summary.user,
            metrics: summary.metrics,
        })
    }

    pub async fn bootstrap_admin(
        &self,
        handle: &str,
        display_name: &str,
        password: &str,
    ) -> anyhow::Result<UserResponse> {
        validate_user_handle(handle)?;
        validate_display_name(display_name)?;
        validate_password(password)?;
        if self.store.user_count().await? > 0 {
            anyhow::bail!("initial admin already exists");
        }
        let password_hash = hash_password(password)?;
        let summary = self
            .store
            .create_user(
                handle,
                display_name,
                &password_hash,
                true,
                &[
                    UserPermission::Read,
                    UserPermission::Write,
                    UserPermission::Repo,
                ],
            )
            .await?;
        Ok(UserResponse {
            user: summary.user,
            metrics: summary.metrics,
        })
    }

    pub async fn update_user(
        &self,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> anyhow::Result<UserResponse> {
        validate_user_handle(&request.handle)?;
        validate_display_name(&request.display_name)?;
        validate_permissions(&request.permissions)?;
        if let Some(existing) = self.store.get_user_by_handle(&request.handle).await?
            && existing.user.id != user_id
        {
            return Err(anyhow::anyhow!(SynforgeError::Conflict(format!(
                "user handle {} already exists",
                request.handle
            ))));
        }
        let summary = self
            .store
            .update_user(
                user_id,
                &request.handle,
                &request.display_name,
                request.active,
                &request.permissions,
            )
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(user_id.to_string())))?;
        Ok(UserResponse {
            user: summary.user,
            metrics: summary.metrics,
        })
    }

    pub async fn change_user_password(
        &self,
        user_id: Uuid,
        request: ChangePasswordRequest,
    ) -> anyhow::Result<()> {
        validate_password(&request.password)?;
        let password_hash = hash_password(&request.password)?;
        let updated = self
            .store
            .update_user_password(user_id, &password_hash)
            .await?;
        if !updated {
            return Err(anyhow::anyhow!(SynforgeError::NotFound(
                user_id.to_string()
            )));
        }
        Ok(())
    }

    pub async fn delete_user(&self, user_id: Uuid) -> anyhow::Result<UserResponse> {
        if self.store.get_user(user_id).await?.is_none() {
            return Err(anyhow::anyhow!(SynforgeError::NotFound(
                user_id.to_string()
            )));
        }
        let user_count = self.store.user_count().await?;
        if user_count <= 1 {
            anyhow::bail!("cannot delete the last user");
        }
        let summary = self
            .store
            .delete_user(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(user_id.to_string())))?;
        Ok(UserResponse {
            user: summary.user,
            metrics: summary.metrics,
        })
    }

    pub async fn get_user_metrics(&self, user_id: Uuid) -> anyhow::Result<UserMetricsResponse> {
        let summary = self
            .store
            .get_user(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(user_id.to_string())))?;
        Ok(UserMetricsResponse {
            metrics: summary.metrics,
        })
    }

    pub async fn increment_user_download_bytes(
        &self,
        user_id: Uuid,
        bytes: u64,
    ) -> anyhow::Result<()> {
        self.store
            .increment_user_download_bytes(user_id, bytes)
            .await
    }

    pub async fn resolve_repo_file_path(
        &self,
        relative_repo_path: &str,
    ) -> anyhow::Result<PathBuf> {
        let requested = normalize_repo_path(relative_repo_path)?;
        let repo_root = self.config.runtime_paths().repo_dir().to_path_buf();
        let path = repo_root.join(&requested);
        if !tokio::fs::try_exists(&path).await? {
            return Err(anyhow::anyhow!(SynforgeError::NotFound(requested)));
        }

        let repo_root = tokio::fs::canonicalize(repo_root).await?;
        let resolved = tokio::fs::canonicalize(&path).await?;
        if !resolved.starts_with(&repo_root) {
            anyhow::bail!("resolved repo path escapes repository root");
        }
        Ok(resolved)
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
            .start_container(
                &container_id,
                None::<bollard::query_parameters::StartContainerOptions>,
            )
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

        let mut wait = docker.wait_container(
            &container_id,
            None::<bollard::query_parameters::WaitContainerOptions>,
        );
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
            anyhow::bail!(
                "cannot delete package {} while a job is pending or running",
                package_name
            );
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
    ) -> anyhow::Result<PackageActionResponse> {
        self.scheduler
            .enqueue_package_action(
                package_name,
                BuildTrigger::ManualRefresh,
                false,
                &self.queue_tx,
            )
            .await
    }

    pub async fn trigger_rebuild(
        &self,
        package_name: &str,
        _request: RebuildRequest,
    ) -> anyhow::Result<PackageActionResponse> {
        self.scheduler
            .enqueue_package_action(
                package_name,
                BuildTrigger::ManualRebuild,
                true,
                &self.queue_tx,
            )
            .await
    }

    pub async fn trigger_target_refresh(
        &self,
        package_name: &str,
        mock_chroot: &str,
        _request: RefreshRequest,
    ) -> anyhow::Result<PackageActionTargetResult> {
        self.scheduler
            .enqueue_target_action(
                package_name,
                mock_chroot,
                BuildTrigger::ManualRefresh,
                false,
                &self.queue_tx,
            )
            .await
    }

    pub async fn trigger_target_rebuild(
        &self,
        package_name: &str,
        mock_chroot: &str,
        _request: RebuildRequest,
    ) -> anyhow::Result<PackageActionTargetResult> {
        self.scheduler
            .enqueue_target_action(
                package_name,
                mock_chroot,
                BuildTrigger::ManualRebuild,
                true,
                &self.queue_tx,
            )
            .await
    }

    pub async fn list_jobs(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<BuildJobListResponse> {
        let (limit, offset) = normalize_pagination(limit, offset);
        let total = self
            .store
            .count_jobs(status, package_name.clone(), mock_chroot.clone(), false)
            .await?;
        let jobs = self
            .store
            .list_jobs(limit, offset, status, package_name, mock_chroot, false)
            .await?;
        Ok(BuildJobListResponse {
            page: build_page_info(limit, offset, total, jobs.len()),
            jobs,
        })
    }

    pub async fn list_completed_jobs(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<BuildJobListResponse> {
        let (limit, offset) = normalize_pagination(limit, offset);
        let total = self
            .store
            .count_jobs(status, package_name.clone(), mock_chroot.clone(), true)
            .await?;
        let jobs = self
            .store
            .list_jobs(limit, offset, status, package_name, mock_chroot, true)
            .await?;
        Ok(BuildJobListResponse {
            page: build_page_info(limit, offset, total, jobs.len()),
            jobs,
        })
    }

    pub async fn list_active_jobs(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<BuildJobListResponse> {
        let (limit, offset) = normalize_pagination(limit, offset);
        let total = self
            .store
            .count_active_jobs(package_name.clone(), mock_chroot.clone())
            .await?;
        let jobs = self
            .store
            .list_active_jobs(limit, offset, package_name, mock_chroot)
            .await?;
        Ok(BuildJobListResponse {
            page: build_page_info(limit, offset, total, jobs.len()),
            jobs,
        })
    }

    pub async fn get_job(&self, job_id: Uuid) -> anyhow::Result<BuildJobResponse> {
        self.store
            .get_job(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))
    }

    pub async fn get_job_artifacts(&self, job_id: Uuid) -> anyhow::Result<JobArtifactListResponse> {
        let job = self.get_job(job_id).await?;
        Ok(JobArtifactListResponse {
            job_id,
            artifacts: job.artifacts,
        })
    }

    pub async fn get_job_artifact_meta(
        &self,
        job_id: Uuid,
        file: &str,
    ) -> anyhow::Result<JobArtifactMetaResponse> {
        let job = self.get_job(job_id).await?;
        let artifact = job
            .artifacts
            .into_iter()
            .find(|artifact| artifact.file == std::path::Path::new(file))
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(file.to_string())))?;
        Ok(JobArtifactMetaResponse { job_id, artifact })
    }

    pub async fn delete_job(&self, job_id: Uuid) -> anyhow::Result<BuildJobResponse> {
        let published_files = self.store.list_published_repo_files_for_job(job_id).await?;
        self.lifecycle
            .remove_published_files(&published_files)
            .await?;
        let deleted = self
            .store
            .delete_job(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))?;
        self.lifecycle.remove_job_runtime(job_id).await?;
        Ok(deleted)
    }

    pub async fn prune_failed_jobs(&self) -> anyhow::Result<PruneJobsResponse> {
        let jobs = self
            .store
            .list_jobs(10_000, 0, None, None, None, false)
            .await?;
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

fn normalize_repo_path(path: &str) -> anyhow::Result<String> {
    let trimmed = path.trim_start_matches('/');
    let normalized = PathBuf::from(trimmed);
    if normalized.as_os_str().is_empty() {
        anyhow::bail!("repository path must not be empty");
    }
    if normalized
        .components()
        .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        anyhow::bail!("repository path contains invalid components");
    }
    Ok(normalized.to_string_lossy().into_owned())
}
