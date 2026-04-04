use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use bollard::Docker;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{CreateContainerOptionsBuilder, LogsOptionsBuilder};
use futures_util::StreamExt;
use serde_json::Value;
use synforge_core::{
    api::{
        BrowseRepositoryRequest, BrowseRepositoryResponse, BuildJobListResponse, BuildJobResponse,
        ChangePasswordRequest, ConfigFieldDescriptor, ConfigFieldType, ConfigSchemaResponse,
        CreatePackageRequest, CreateUserRequest, EffectiveConfigDto, EffectiveConfigView,
        LogChunkResponse, LogManifestResponse, LogMetaResponse, LogSource, LogSourceType,
        MockChrootListResponse, PackageActionResponse, PackageBuildHistoryResponse,
        PackageBuildInventoryEntry, PackageListResponse, PackageRepoFilesResponse,
        PackageResponse, PageInfo, PruneJobsResponse, RebuildRequest, RefreshRequest,
        RepoInventoryResponse,
        RepoSummaryResponse, SessionResponse, SetupInitializeRequest, UpdatePackageRequest,
        UpdateRuntimeSettingsRequest, UpdateUserRequest, UserListResponse, UserMetricsResponse,
        UserResponse,
    },
    config::DaemonConfig,
    error::SynforgeError,
    model::{BuildStatus, BuildTrigger, PublishedRepoFile, UserAccount, UserPermission},
    package::parse_mock_chroot,
};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
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
    pub async fn config_schema(&self) -> ConfigSchemaResponse {
        ConfigSchemaResponse {
            fields: editable_config_fields(),
        }
    }

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

    async fn resolve_job_log_path(
        &self,
        job_id: Uuid,
        source: &str,
    ) -> anyhow::Result<std::path::PathBuf> {
        let row = self
            .store
            .get_build_log_for_job_source(job_id, source)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(SynforgeError::NotFound(format!(
                    "log source {} for job {}",
                    source, job_id
                )))
            })?;
        let path = PathBuf::from(row.log_path);
        if tokio::fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(path);
        }
        Err(anyhow::anyhow!(SynforgeError::NotFound(
            path.display().to_string()
        )))
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
            .find(|artifact| artifact.path == std::path::Path::new(relative_artifact_path))
            .ok_or_else(|| {
                anyhow::anyhow!(SynforgeError::NotFound(relative_artifact_path.to_string()))
            })?;

        let path = self
            .config
            .runtime_paths()
            .job_artifacts_dir(job_id)
            .join(&artifact.path);
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
        let runner = BuildRunner::new(
            config.clone(),
            worker_launcher.clone(),
            lifecycle.clone(),
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

    pub async fn effective_config(&self) -> EffectiveConfigDto {
        let current = DaemonConfig::load().unwrap_or_else(|_| self.config.clone());
        let paths = current.runtime_paths();
        EffectiveConfigDto {
            config: EffectiveConfigView {
                config_path: DaemonConfig::config_path(),
                bootstrap_completed: current.bootstrap_completed,
                listen_addr: current.listen_addr.clone(),
                runtime_root: current.runtime_root.clone(),
                database_url: current.database_url.clone(),
                packages_dir: paths.packages_dir().to_path_buf(),
                repo_dir: paths.repo_dir().to_path_buf(),
                jobs_root: paths.jobs_root().to_path_buf(),
                worker_image: current.worker_image.clone(),
                max_concurrent_builds: current.max_concurrent_builds,
                db_pool_size: current.db_pool_size,
                queue_buffer_size: current.queue_buffer_size,
                poller_tick_seconds: current.poller_tick_seconds,
                worker_result_timeout_seconds: current.worker_result_timeout_seconds,
                worker_socket_timeout_seconds: current.worker_socket_timeout_seconds,
                git_operation_timeout_seconds: current.git_operation_timeout_seconds,
                public_base_url: current.public_base_url,
            },
        }
    }

    pub async fn update_runtime_settings(
        &self,
        request: UpdateRuntimeSettingsRequest,
    ) -> anyhow::Result<EffectiveConfigDto> {
        let mut config = DaemonConfig::load()?;
        apply_config_settings(&mut config, &request.settings, false)?;
        config.save()?;
        Ok(self.effective_config().await)
    }

    pub async fn initialize_setup(
        &self,
        request: SetupInitializeRequest,
    ) -> anyhow::Result<EffectiveConfigDto> {
        let current = DaemonConfig::load()?;
        if current.bootstrap_completed {
            anyhow::bail!("setup has already been completed");
        }
        validate_user_handle(&request.admin.handle)?;
        validate_display_name(&request.admin.display_name)?;
        validate_password(&request.admin.password)?;
        let mut config = current;
        apply_config_settings(&mut config, &request.settings, true)?;
        config.bootstrap_completed = false;
        config
            .validate()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        if self.store.user_count().await? > 0 {
            anyhow::bail!("initial admin cannot be created because users already exist");
        }
        config.save()?;
        self.bootstrap_admin(
            &request.admin.handle,
            &request.admin.display_name,
            &request.admin.password,
        )
        .await?;
        config.bootstrap_completed = true;
        config.save()?;
        Ok(self.effective_config().await)
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
    ) -> anyhow::Result<PackageBuildHistoryResponse> {
        self.registry.get_package(package_name).await?;
        let jobs = self.store.list_jobs_for_package(package_name).await?;
        let published_files = self
            .store
            .list_published_repo_files_for_package(package_name)
            .await?;
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
            .count_jobs(status, package_name.clone(), mock_chroot.clone())
            .await?;
        let jobs = self
            .store
            .list_jobs(limit, offset, status, package_name, mock_chroot)
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
        let mut sources = Vec::new();
        let db_logs = self.store.list_build_logs_for_job(job_id).await?;

        for row in db_logs {
            let log_path = PathBuf::from(&row.log_path);
            if let Ok(meta) = tokio::fs::metadata(&log_path).await {
                sources.push(LogSource {
                    path: row.source_path,
                    size: meta.len(),
                    source_type: LogSourceType::Raw,
                });
            }
        }

        Ok(LogManifestResponse { job_id, sources })
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
        let jobs = self.store.list_jobs(10_000, 0, None, None, None).await?;
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

#[derive(Copy, Clone)]
struct ConfigSection<'a> {
    key: &'a str,
    label: &'a str,
}

#[derive(Copy, Clone)]
struct ConfigEditability {
    in_setup: bool,
    in_runtime: bool,
}

fn editable_config_fields() -> Vec<ConfigFieldDescriptor> {
    vec![
        config_string_field(
            ConfigSection {
                key: "server",
                label: "Server",
            },
            "listen_addr",
            "Listen address",
            "Daemon HTTP listen address.",
            "0.0.0.0:8080",
            ConfigEditability {
                in_setup: true,
                in_runtime: false,
            },
        ),
        config_string_field(
            ConfigSection {
                key: "storage",
                label: "Storage",
            },
            "runtime_root",
            "Runtime root",
            "Root directory for database, package metadata, repo files, and jobs.",
            "/var/lib/synforge",
            ConfigEditability {
                in_setup: true,
                in_runtime: false,
            },
        ),
        config_string_field(
            ConfigSection {
                key: "server",
                label: "Server",
            },
            "public_base_url",
            "Public base URL",
            "Base URL used in generated links and repo setup.",
            "http://localhost:8080",
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_string_field(
            ConfigSection {
                key: "worker",
                label: "Worker",
            },
            "worker_image",
            "Worker image",
            "Docker image used for spawned worker containers.",
            "synforge-worker-fedora:latest",
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "build",
                label: "Build",
            },
            "max_concurrent_builds",
            "Max concurrent builds",
            "Maximum number of active builds at once.",
            2,
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "database",
                label: "Database",
            },
            "db_pool_size",
            "DB pool size",
            "Number of database connection pool slots.",
            5,
            ConfigEditability {
                in_setup: true,
                in_runtime: false,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "scheduler",
                label: "Scheduler",
            },
            "queue_buffer_size",
            "Queue buffer size",
            "In-memory queued build channel capacity.",
            128,
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "scheduler",
                label: "Scheduler",
            },
            "poller_tick_seconds",
            "Poller tick seconds",
            "How often package polling wakes up.",
            30,
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "worker",
                label: "Worker",
            },
            "worker_result_timeout_seconds",
            "Worker result timeout seconds",
            "Timeout while waiting for worker completion after request dispatch.",
            10,
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "worker",
                label: "Worker",
            },
            "worker_socket_timeout_seconds",
            "Worker socket timeout seconds",
            "Socket timeout used for worker protocol I/O.",
            30,
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "git",
                label: "Git",
            },
            "git_operation_timeout_seconds",
            "Git operation timeout seconds",
            "Timeout applied to git inspection and sync commands.",
            600,
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
    ]
}

fn config_string_field(
    section: ConfigSection<'_>,
    key: &str,
    label: &str,
    description: &str,
    default_value: &str,
    editability: ConfigEditability,
) -> ConfigFieldDescriptor {
    ConfigFieldDescriptor {
        key: key.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        section_key: section.key.to_string(),
        section_label: section.label.to_string(),
        field_type: ConfigFieldType::String,
        required: true,
        min_value: None,
        editable_in_setup: editability.in_setup,
        editable_in_runtime: editability.in_runtime,
        default_value: Value::String(default_value.to_string()),
    }
}

fn config_number_field(
    section: ConfigSection<'_>,
    key: &str,
    label: &str,
    description: &str,
    default_value: u64,
    editability: ConfigEditability,
) -> ConfigFieldDescriptor {
    ConfigFieldDescriptor {
        key: key.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        section_key: section.key.to_string(),
        section_label: section.label.to_string(),
        field_type: ConfigFieldType::Number,
        required: true,
        min_value: Some(1),
        editable_in_setup: editability.in_setup,
        editable_in_runtime: editability.in_runtime,
        default_value: Value::Number(default_value.into()),
    }
}

fn apply_config_settings(
    config: &mut DaemonConfig,
    settings: &BTreeMap<String, Value>,
    allow_setup_only: bool,
) -> anyhow::Result<()> {
    for key in settings.keys() {
        let Some(field) = editable_config_fields()
            .into_iter()
            .find(|field| field.key == *key)
        else {
            anyhow::bail!("unknown config setting: {key}");
        };
        if allow_setup_only {
            if !field.editable_in_setup {
                anyhow::bail!("config setting is not editable during setup: {key}");
            }
        } else if !field.editable_in_runtime {
            anyhow::bail!("config setting is not editable at runtime: {key}");
        }
    }

    if let Some(value) = settings.get("listen_addr") {
        config.listen_addr = parse_string_setting(value, "listen_addr")?;
    }
    if let Some(value) = settings.get("runtime_root") {
        config.runtime_root = PathBuf::from(parse_string_setting(value, "runtime_root")?);
    }
    if let Some(value) = settings.get("public_base_url") {
        config.public_base_url = parse_string_setting(value, "public_base_url")?;
    }
    if let Some(value) = settings.get("worker_image") {
        config.worker_image = parse_string_setting(value, "worker_image")?;
    }
    if let Some(value) = settings.get("max_concurrent_builds") {
        config.max_concurrent_builds = parse_usize_setting(value, "max_concurrent_builds")?;
    }
    if let Some(value) = settings.get("db_pool_size") {
        config.db_pool_size = parse_u32_setting(value, "db_pool_size")?;
    }
    if let Some(value) = settings.get("queue_buffer_size") {
        config.queue_buffer_size = parse_usize_setting(value, "queue_buffer_size")?;
    }
    if let Some(value) = settings.get("poller_tick_seconds") {
        config.poller_tick_seconds = parse_u64_setting(value, "poller_tick_seconds")?;
    }
    if let Some(value) = settings.get("worker_result_timeout_seconds") {
        config.worker_result_timeout_seconds =
            parse_u64_setting(value, "worker_result_timeout_seconds")?;
    }
    if let Some(value) = settings.get("worker_socket_timeout_seconds") {
        config.worker_socket_timeout_seconds =
            parse_u64_setting(value, "worker_socket_timeout_seconds")?;
    }
    if let Some(value) = settings.get("git_operation_timeout_seconds") {
        config.git_operation_timeout_seconds =
            parse_u64_setting(value, "git_operation_timeout_seconds")?;
    }

    Ok(())
}

fn parse_string_setting(value: &Value, key: &str) -> anyhow::Result<String> {
    let Some(value) = value.as_str() else {
        anyhow::bail!("config setting must be a string: {key}");
    };
    let value = value.trim().to_string();
    if value.is_empty() {
        anyhow::bail!("config setting must not be empty: {key}");
    }
    Ok(value)
}

fn parse_u64_setting(value: &Value, key: &str) -> anyhow::Result<u64> {
    let Some(value) = value.as_u64() else {
        anyhow::bail!("config setting must be a positive integer: {key}");
    };
    if value == 0 {
        anyhow::bail!("config setting must be greater than zero: {key}");
    }
    Ok(value)
}

fn parse_usize_setting(value: &Value, key: &str) -> anyhow::Result<usize> {
    Ok(parse_u64_setting(value, key)? as usize)
}

fn parse_u32_setting(value: &Value, key: &str) -> anyhow::Result<u32> {
    u32::try_from(parse_u64_setting(value, key)?)
        .map_err(|_| anyhow::anyhow!("config setting is out of range: {key}"))
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
