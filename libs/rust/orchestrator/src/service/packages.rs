use bollard::Docker;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{CreateContainerOptionsBuilder, LogsOptionsBuilder};
use futures_util::StreamExt;
use synforge_core::{
    api::{
        BrowseRepositoryRequest, BrowseRepositoryResponse, CreatePackageRequest,
        MockChrootListResponse, PackageActionResponse, PackageActionTargetResult,
        PackageBuildHistoryResponse, PackageBuildInventoryEntry, PackageListResponse,
        PackageResponse, RebuildRequest, RefreshRequest, UpdatePackageRequest,
    },
    model::{BuildStatus, BuildTrigger, PublishedRepoFile},
    package::parse_mock_chroot,
};
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
}
