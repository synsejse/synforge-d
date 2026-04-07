use std::time::Duration;

use bollard::Docker;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{CreateContainerOptionsBuilder, LogsOptionsBuilder};
use futures_util::StreamExt;
use synforge_core::{
    api::{
        BrowseRepositoryProgressResponse, BrowseRepositoryRequest, BrowseRepositoryResponse,
        CreatePackageRequest, MockChrootListResponse, PackageActionResponse,
        PackageActionTargetResult, PackageBuildHistoryResponse, PackageBuildInventoryEntry,
        PackageListResponse, PackageResponse, RebuildRequest, RefreshAllPackagesProgressResponse,
        RefreshAllPackagesProgressView, RefreshAllPackagesResponse, RefreshAllPackagesState,
        RefreshRequest, UpdatePackageRequest,
    },
    error::SynforgeError,
    model::{BuildStatus, BuildTrigger, PublishedRepoFile, now_utc},
    package::parse_mock_chroot,
};
use tracing::{info, warn};
use uuid::Uuid;

use super::SynforgeService;
use crate::db::{JobStore, PackageStore, RepoStore};

impl SynforgeService {
    pub async fn list_packages(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<PackageListResponse> {
        let (limit, offset) = super::normalize_pagination(limit, offset);
        let total = self.store.count_packages(search.clone(), enabled).await?;
        let packages = self
            .store
            .list_packages(limit, offset, search, enabled)
            .await?;
        Ok(PackageListResponse {
            page: super::build_page_info(limit, offset, total, packages.len()),
            packages,
        })
    }

    pub async fn get_package(&self, package_name: &str) -> anyhow::Result<PackageResponse> {
        self.registry.get_package(package_name).await
    }

    pub async fn get_refresh_all_packages_progress(
        &self,
    ) -> anyhow::Result<RefreshAllPackagesProgressResponse> {
        let progress = self.refresh_all_packages_progress.lock().await.clone();
        Ok(RefreshAllPackagesProgressResponse {
            operation: progress,
        })
    }

    pub async fn trigger_refresh_all_packages(&self) -> anyhow::Result<RefreshAllPackagesResponse> {
        let operation_id = Uuid::now_v7();
        let mut progress = RefreshAllPackagesProgressView {
            operation_id,
            state: RefreshAllPackagesState::Running,
            total_packages: 0,
            processed_packages: 0,
            queued_packages: 0,
            skipped_packages: 0,
            blocked_packages: 0,
            failed_packages: 0,
            queued_targets: 0,
            skipped_targets: 0,
            blocked_targets: 0,
            message: Some("collecting enabled packages".to_string()),
        };
        {
            let mut slot = self.refresh_all_packages_progress.lock().await;
            if let Some(operation) = slot.as_ref()
                && operation.state == RefreshAllPackagesState::Running
            {
                return Err(anyhow::anyhow!(SynforgeError::Conflict(
                    "refresh-all operation is already running".to_string()
                )));
            }
            *slot = Some(progress.clone());
        }

        let package_names = match self.list_all_enabled_package_names().await {
            Ok(package_names) => package_names,
            Err(error) => {
                progress.state = RefreshAllPackagesState::Failed;
                progress.message = Some(error.to_string());
                self.update_refresh_all_packages_progress(progress).await;
                return Err(error);
            }
        };

        progress.total_packages = package_names.len() as u64;
        progress.message = None;
        self.update_refresh_all_packages_progress(progress.clone())
            .await;

        if package_names.is_empty() {
            progress.state = RefreshAllPackagesState::Completed;
            progress.message = Some("no enabled packages found to refresh".to_string());
            self.update_refresh_all_packages_progress(progress.clone())
                .await;
            return Ok(RefreshAllPackagesResponse {
                operation: progress,
            });
        }

        for package_name in package_names {
            match self
                .scheduler
                .enqueue_package_action(
                    &package_name,
                    BuildTrigger::ManualRefresh,
                    false,
                    &self.queue_tx,
                )
                .await
            {
                Ok(response) => {
                    let mut package_has_queued = false;
                    let mut package_has_skipped = false;
                    let mut package_has_blocked = false;
                    for result in response.results {
                        match result.disposition {
                            synforge_core::api::PackageActionDisposition::Queued => {
                                progress.queued_targets += 1;
                                package_has_queued = true;
                            }
                            synforge_core::api::PackageActionDisposition::Skipped => {
                                progress.skipped_targets += 1;
                                package_has_skipped = true;
                            }
                            synforge_core::api::PackageActionDisposition::Blocked => {
                                progress.blocked_targets += 1;
                                package_has_blocked = true;
                            }
                        }
                    }

                    if package_has_queued {
                        progress.queued_packages += 1;
                    } else if package_has_blocked {
                        progress.blocked_packages += 1;
                    } else if package_has_skipped {
                        progress.skipped_packages += 1;
                    } else {
                        progress.skipped_packages += 1;
                    }
                }
                Err(error) => {
                    progress.failed_packages += 1;
                    if progress.message.is_none() {
                        progress.message = Some(format!(
                            "first failed package: {} ({})",
                            package_name, error
                        ));
                    }
                    warn!(
                        package_name,
                        error = %error,
                        "refresh-all package action failed"
                    );
                }
            }

            progress.processed_packages += 1;
            self.update_refresh_all_packages_progress(progress.clone())
                .await;
        }

        progress.state = RefreshAllPackagesState::Completed;
        if progress.failed_packages == 0 {
            progress.message = None;
        } else {
            progress.message = Some(format!(
                "{} package(s) failed to refresh",
                progress.failed_packages
            ));
        }
        self.update_refresh_all_packages_progress(progress.clone())
            .await;
        Ok(RefreshAllPackagesResponse {
            operation: progress,
        })
    }

    pub async fn get_package_build_history(
        &self,
        package_name: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> anyhow::Result<PackageBuildHistoryResponse> {
        self.registry.get_package(package_name).await?;
        let (limit, offset) = super::normalize_pagination(limit, offset);
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
            page: super::build_page_info(limit, offset, total, returned),
        })
    }

    pub async fn create_package(
        &self,
        request: CreatePackageRequest,
    ) -> anyhow::Result<PackageResponse> {
        self.registry.create_package(request).await
    }

    pub async fn list_mock_chroots(&self) -> anyhow::Result<MockChrootListResponse> {
        let ttl = Duration::from_secs(self.config.mock_chroot_cache_ttl_seconds);
        let now = std::time::Instant::now();
        let mut cache = self.mock_chroot_cache.lock().await;
        let cache_hit = cache.entry.as_ref().and_then(|entry| {
            if entry.worker_image == self.config.worker_image
                && now.duration_since(entry.fetched_at) < ttl
            {
                Some(entry.response.clone())
            } else {
                None
            }
        });
        if let Some(response) = cache_hit {
            cache.hit_count = cache.hit_count.saturating_add(1);
            return Ok(response);
        }
        cache.miss_count = cache.miss_count.saturating_add(1);
        let stale = cache.entry.clone();
        match self.load_mock_chroots_uncached().await {
            Ok(response) => {
                cache.entry = Some(super::MockChrootCacheEntry {
                    worker_image: self.config.worker_image.clone(),
                    fetched_at: now,
                    fetched_at_unix_seconds: now_utc().unix_timestamp(),
                    response: response.clone(),
                });
                Ok(response)
            }
            Err(error) => {
                if let Some(entry) = stale
                    && entry.worker_image == self.config.worker_image
                {
                    cache.stale_served_count = cache.stale_served_count.saturating_add(1);
                    warn!(
                        worker_image = %self.config.worker_image,
                        error = %error,
                        stale_count = entry.response.chroots.len(),
                        "failed to refresh mock chroot cache; serving stale value"
                    );
                    return Ok(entry.response);
                }
                Err(error)
            }
        }
    }

    async fn load_mock_chroots_uncached(&self) -> anyhow::Result<MockChrootListResponse> {
        let docker = Docker::connect_with_local_defaults()?;
        let container_name = format!("synforge-mock-chroots-{}", Uuid::now_v7());
        info!(
            container_name = %container_name,
            worker_image = %self.config.worker_image,
            "listing mock chroots via worker image"
        );
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
        info!(container_id = %container_id, "starting mock chroot listing container");
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
        info!(
            container_id = %container_id,
            chroot_count = chroots.len(),
            "mock chroot listing completed"
        );
        Ok(MockChrootListResponse { chroots })
    }

    pub async fn browse_repository(
        &self,
        request: BrowseRepositoryRequest,
    ) -> anyhow::Result<BrowseRepositoryResponse> {
        self.registry.browse_repository(&request.repo_url).await
    }

    pub async fn get_browse_repository_progress(
        &self,
    ) -> anyhow::Result<BrowseRepositoryProgressResponse> {
        Ok(BrowseRepositoryProgressResponse {
            operation: self.registry.browse_repository_progress().await,
        })
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
        info!(package_name, trigger = ?BuildTrigger::ManualRefresh, "manual refresh requested");
        let response = self
            .scheduler
            .enqueue_package_action(
                package_name,
                BuildTrigger::ManualRefresh,
                false,
                &self.queue_tx,
            )
            .await?;
        log_action_response(package_name, BuildTrigger::ManualRefresh, &response.results);
        Ok(response)
    }

    pub async fn trigger_rebuild(
        &self,
        package_name: &str,
        _request: RebuildRequest,
    ) -> anyhow::Result<PackageActionResponse> {
        info!(package_name, trigger = ?BuildTrigger::ManualRebuild, "manual rebuild requested");
        let response = self
            .scheduler
            .enqueue_package_action(
                package_name,
                BuildTrigger::ManualRebuild,
                true,
                &self.queue_tx,
            )
            .await?;
        log_action_response(package_name, BuildTrigger::ManualRebuild, &response.results);
        Ok(response)
    }

    pub async fn trigger_target_refresh(
        &self,
        package_name: &str,
        mock_chroot: &str,
        _request: RefreshRequest,
    ) -> anyhow::Result<PackageActionTargetResult> {
        info!(
            package_name,
            mock_chroot,
            trigger = ?BuildTrigger::ManualRefresh,
            "manual target refresh requested"
        );
        let result = self
            .scheduler
            .enqueue_target_action(
                package_name,
                mock_chroot,
                BuildTrigger::ManualRefresh,
                false,
                &self.queue_tx,
            )
            .await?;
        info!(
            package_name,
            mock_chroot,
            trigger = ?BuildTrigger::ManualRefresh,
            disposition = ?result.disposition,
            "manual target refresh scheduled"
        );
        Ok(result)
    }

    pub async fn trigger_target_rebuild(
        &self,
        package_name: &str,
        mock_chroot: &str,
        _request: RebuildRequest,
    ) -> anyhow::Result<PackageActionTargetResult> {
        info!(
            package_name,
            mock_chroot,
            trigger = ?BuildTrigger::ManualRebuild,
            "manual target rebuild requested"
        );
        let result = self
            .scheduler
            .enqueue_target_action(
                package_name,
                mock_chroot,
                BuildTrigger::ManualRebuild,
                true,
                &self.queue_tx,
            )
            .await?;
        info!(
            package_name,
            mock_chroot,
            trigger = ?BuildTrigger::ManualRebuild,
            disposition = ?result.disposition,
            "manual target rebuild scheduled"
        );
        Ok(result)
    }

    async fn list_all_enabled_package_names(&self) -> anyhow::Result<Vec<String>> {
        const PAGE_SIZE: usize = 200;
        let mut package_names = Vec::new();
        let mut offset = 0;

        loop {
            let page = self
                .store
                .list_packages(PAGE_SIZE, offset, None, Some(true))
                .await?;
            if page.is_empty() {
                break;
            }
            offset += page.len();
            package_names.extend(page.into_iter().map(|entry| entry.package.name));
        }

        Ok(package_names)
    }

    async fn update_refresh_all_packages_progress(&self, progress: RefreshAllPackagesProgressView) {
        let mut slot = self.refresh_all_packages_progress.lock().await;
        *slot = Some(progress);
    }
}

fn log_action_response(
    package_name: &str,
    trigger: BuildTrigger,
    results: &[PackageActionTargetResult],
) {
    let mut queued = 0_usize;
    let mut skipped = 0_usize;
    let mut blocked = 0_usize;
    for result in results {
        match result.disposition {
            synforge_core::api::PackageActionDisposition::Queued => queued += 1,
            synforge_core::api::PackageActionDisposition::Skipped => skipped += 1,
            synforge_core::api::PackageActionDisposition::Blocked => blocked += 1,
        }
    }
    info!(
        package_name,
        trigger = ?trigger,
        queued_targets = queued,
        skipped_targets = skipped,
        blocked_targets = blocked,
        "manual package action scheduled"
    );
}
