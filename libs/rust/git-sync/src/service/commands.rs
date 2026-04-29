//! Command-side operations for git synchronization.

use async_trait::async_trait;
use synforge_core::{
    api::{
        BuildJobResponse, CreatePackageRequest, PackageActionDisposition, PackageActionResponse,
        PackageResponse, RefreshAllPackagesProgressView, RefreshAllPackagesResponse,
        RefreshAllPackagesState, UpdatePackageRequest,
    },
    error::SynforgeError,
    model::BuildStatus,
    package::{PackageDefinition, SpecSource},
};
use tracing::warn;
use uuid::Uuid;

use super::{
    queries::{PackageDetailsReader, get_refresh_all_packages_progress},
    state::{InspectedPackageSource, PackageMaterializationOptions},
};

#[async_trait]
pub trait EnabledPackageCatalog {
    async fn list_all_enabled_package_names(&self) -> anyhow::Result<Vec<String>>;
}

#[async_trait]
pub trait ManualRefreshScheduler {
    async fn enqueue_manual_refresh(
        &self,
        package_name: &str,
    ) -> anyhow::Result<PackageActionResponse>;
}

#[async_trait]
pub trait PackageLookup {
    async fn find_package(&self, package_name: &str) -> anyhow::Result<Option<PackageResponse>>;
}

#[async_trait]
pub trait PackageDefinitionWriter {
    async fn upsert_package_definition(&self, package: &PackageDefinition) -> anyhow::Result<()>;
}

#[async_trait]
pub trait PackageSourceInspector {
    async fn inspect_source(
        &self,
        package_name: &str,
        source: &SpecSource,
        timeout_seconds: u64,
    ) -> anyhow::Result<InspectedPackageSource>;
}

#[async_trait]
pub trait PackageDefinitionMaterializer {
    async fn materialize_package_definition(
        &self,
        source: &SpecSource,
        inspected: &InspectedPackageSource,
        options: PackageMaterializationOptions,
    ) -> anyhow::Result<PackageDefinition>;
}

#[async_trait]
pub trait PackageDeletionJobReader {
    async fn list_jobs_for_package(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Vec<BuildJobResponse>>;
}

#[async_trait]
pub trait PackageDeletionRunner {
    async fn delete_package_job(&self, job_id: Uuid) -> anyhow::Result<()>;
}

#[async_trait]
pub trait PackageDeleter {
    async fn delete_package(&self, package_name: &str) -> anyhow::Result<()>;
}

#[async_trait]
pub trait RefreshAllProgressStore {
    async fn load_refresh_all_packages_progress(&self) -> Option<RefreshAllPackagesProgressView>;

    async fn save_refresh_all_packages_progress(&self, progress: RefreshAllPackagesProgressView);
}

#[tracing::instrument(skip_all, fields(operation_id))]
pub async fn trigger_refresh_all_packages<D>(deps: &D) -> anyhow::Result<RefreshAllPackagesResponse>
where
    D: EnabledPackageCatalog + ManualRefreshScheduler + RefreshAllProgressStore + Send + Sync,
{
    let operation_id = Uuid::now_v7();
    tracing::Span::current().record("operation_id", tracing::field::display(&operation_id));
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

    let existing = get_refresh_all_packages_progress(deps).await?;
    if existing
        .operation
        .as_ref()
        .is_some_and(|operation| operation.state == RefreshAllPackagesState::Running)
    {
        return Err(anyhow::anyhow!(SynforgeError::Conflict(
            "refresh-all operation is already running".to_string()
        )));
    }

    deps.save_refresh_all_packages_progress(progress.clone())
        .await;

    let package_names = match deps.list_all_enabled_package_names().await {
        Ok(package_names) => package_names,
        Err(error) => {
            progress.state = RefreshAllPackagesState::Failed;
            progress.message = Some(error.to_string());
            deps.save_refresh_all_packages_progress(progress.clone())
                .await;
            return Err(error);
        }
    };

    progress.total_packages = package_names.len() as u64;
    progress.message = None;
    deps.save_refresh_all_packages_progress(progress.clone())
        .await;

    if package_names.is_empty() {
        progress.state = RefreshAllPackagesState::Completed;
        progress.message = Some("no enabled packages found to refresh".to_string());
        deps.save_refresh_all_packages_progress(progress.clone())
            .await;
        return Ok(RefreshAllPackagesResponse {
            operation: progress,
        });
    }

    for package_name in package_names {
        match deps.enqueue_manual_refresh(&package_name).await {
            Ok(response) => {
                let mut package_has_queued = false;
                let mut package_has_blocked = false;
                for result in response.results {
                    match result.disposition {
                        PackageActionDisposition::Queued => {
                            progress.queued_targets += 1;
                            package_has_queued = true;
                        }
                        PackageActionDisposition::Skipped => {
                            progress.skipped_targets += 1;
                        }
                        PackageActionDisposition::Blocked => {
                            progress.blocked_targets += 1;
                            package_has_blocked = true;
                        }
                    }
                }

                if package_has_queued {
                    progress.queued_packages += 1;
                } else if package_has_blocked {
                    progress.blocked_packages += 1;
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
        deps.save_refresh_all_packages_progress(progress.clone())
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
    deps.save_refresh_all_packages_progress(progress.clone())
        .await;

    Ok(RefreshAllPackagesResponse {
        operation: progress,
    })
}

#[tracing::instrument(skip(deps), fields(package_name = %package_name))]
pub async fn delete_package<D>(deps: &D, package_name: &str) -> anyhow::Result<()>
where
    D: PackageDeletionJobReader + PackageDeletionRunner + PackageDeleter + Send + Sync,
{
    let jobs = deps.list_jobs_for_package(package_name).await?;
    if jobs.iter().any(|entry| {
        matches!(
            entry.job.status,
            BuildStatus::Pending | BuildStatus::Running
        )
    }) {
        return Err(anyhow::anyhow!(SynforgeError::Conflict(format!(
            "cannot delete package {} while a job is pending or running",
            package_name
        ))));
    }

    for job in jobs {
        deps.delete_package_job(job.job.id).await?;
    }

    deps.delete_package(package_name).await
}

#[tracing::instrument(skip(deps), fields(package_name = %request.name))]
pub async fn create_package<D>(
    deps: &D,
    request: CreatePackageRequest,
) -> anyhow::Result<PackageResponse>
where
    D: PackageLookup
        + PackageDefinitionWriter
        + PackageSourceInspector
        + PackageDefinitionMaterializer
        + PackageDetailsReader
        + Send
        + Sync,
{
    if deps.find_package(&request.name).await?.is_some() {
        return Err(anyhow::anyhow!(SynforgeError::Conflict(format!(
            "package {} already exists",
            request.name
        ))));
    }

    let inspected = deps
        .inspect_source(
            &request.name,
            &request.source,
            request.build_timeout_seconds,
        )
        .await?;
    let package = deps
        .materialize_package_definition(
            &request.source,
            &inspected,
            PackageMaterializationOptions {
                enabled: request.enabled,
                publish_srpm: request.publish_srpm,
                publish_debuginfo: request.publish_debuginfo,
                network_access: request.network_access,
                ccache_enabled: request.ccache_enabled,
                ccache_max_size_mb: positive_limit(request.ccache_max_size_mb),
                mock_chroots: request.mock_chroots,
                poll_interval_seconds: request.poll_interval_seconds,
                build_timeout_seconds: request.build_timeout_seconds,
                package_history_count: request.package_history_count,
                cpu_limit_millicores: positive_limit(request.cpu_limit_millicores),
                memory_limit_mb: positive_limit(request.memory_limit_mb),
                build_env: request.build_env,
            },
        )
        .await?;
    deps.upsert_package_definition(&package).await?;
    deps.get_package(&request.name).await
}

#[tracing::instrument(skip(deps, request), fields(package_name = %package_name))]
pub async fn update_package<D>(
    deps: &D,
    package_name: &str,
    request: UpdatePackageRequest,
) -> anyhow::Result<PackageResponse>
where
    D: PackageDefinitionWriter
        + PackageSourceInspector
        + PackageDefinitionMaterializer
        + PackageDetailsReader
        + Send
        + Sync,
{
    let existing = deps.get_package(package_name).await?;
    let build_timeout_seconds = request
        .build_timeout_seconds
        .unwrap_or(existing.package.build_timeout_seconds);
    let inspected = deps
        .inspect_source(package_name, &request.source, build_timeout_seconds)
        .await?;
    let package = deps
        .materialize_package_definition(
            &request.source,
            &inspected,
            PackageMaterializationOptions {
                enabled: request.enabled.unwrap_or(existing.package.enabled),
                publish_srpm: request
                    .publish_srpm
                    .unwrap_or(existing.package.publish_srpm),
                publish_debuginfo: request
                    .publish_debuginfo
                    .unwrap_or(existing.package.publish_debuginfo),
                network_access: request
                    .network_access
                    .unwrap_or(existing.package.network_access),
                ccache_enabled: request
                    .ccache_enabled
                    .unwrap_or(existing.package.ccache_enabled),
                ccache_max_size_mb: merge_optional_limit(
                    request.ccache_max_size_mb,
                    existing.package.ccache_max_size_mb,
                ),
                mock_chroots: request
                    .mock_chroots
                    .unwrap_or_else(|| existing.package.mock_chroots.clone()),
                poll_interval_seconds: request
                    .poll_interval_seconds
                    .unwrap_or(existing.package.poll_interval_seconds),
                build_timeout_seconds,
                package_history_count: request
                    .package_history_count
                    .unwrap_or(existing.package.package_history_count),
                cpu_limit_millicores: merge_optional_limit(
                    request.cpu_limit_millicores,
                    existing.package.cpu_limit_millicores,
                ),
                memory_limit_mb: merge_optional_limit(
                    request.memory_limit_mb,
                    existing.package.memory_limit_mb,
                ),
                build_env: request
                    .build_env
                    .unwrap_or_else(|| existing.package.build_env.clone()),
            },
        )
        .await?;
    if package.name != package_name {
        return Err(anyhow::anyhow!(SynforgeError::BadRequest(
            "updated package name must match existing package".to_string(),
        )));
    }
    deps.upsert_package_definition(&package).await?;
    deps.get_package(package_name).await
}

fn positive_limit(value: Option<u64>) -> Option<u64> {
    value.filter(|value| *value > 0)
}

fn merge_optional_limit(requested: Option<u64>, existing: Option<u64>) -> Option<u64> {
    match requested {
        Some(0) => None,
        Some(value) => Some(value),
        None => existing,
    }
}
