use synforge_core::{
    api::{PackageActionResponse, PackageActionTargetResult},
    error::SynforgeError,
    model::BuildTrigger,
    sync::SyncStage,
};
use synforge_git_sync::{PackageDefinitionMaterializer, PackageDefinitionWriter};
use uuid::Uuid;

use super::{prepare_package_action, report_stage, reserve_and_enqueue_builds};
use crate::service::commands::{
    ActiveTargetBuildReader, BuildJobWriter, BuildQueue, LastSuccessfulRevisionReader,
    PackageDefinitionReader, SyncRunReporter, TargetBuildBackoffReader, TrackedSourceInspector,
};

pub async fn trigger_package_action<D>(
    deps: &D,
    package_name: &str,
    trigger: BuildTrigger,
    force: bool,
) -> anyhow::Result<PackageActionResponse>
where
    D: PackageActionDependencies,
{
    trigger_package_action_inner(deps, package_name, trigger, force, None).await
}

pub async fn trigger_package_action_for_sync<D>(
    deps: &D,
    package_name: &str,
    trigger: BuildTrigger,
    force: bool,
    sync_operation_id: Uuid,
) -> anyhow::Result<PackageActionResponse>
where
    D: PackageActionDependencies,
{
    trigger_package_action_inner(deps, package_name, trigger, force, Some(sync_operation_id)).await
}

async fn trigger_package_action_inner<D>(
    deps: &D,
    package_name: &str,
    trigger: BuildTrigger,
    force: bool,
    sync_operation_id: Option<Uuid>,
) -> anyhow::Result<PackageActionResponse>
where
    D: PackageActionDependencies,
{
    let package = deps.get_package_definition(package_name).await?;
    let plan =
        prepare_package_action(deps, package, None, trigger, force, sync_operation_id).await?;
    report_stage(
        deps,
        sync_operation_id,
        SyncStage::EnqueuingBuilds,
        "Reserving and enqueuing builds",
    )
    .await?;
    let plan = reserve_and_enqueue_builds(deps, plan).await?;

    Ok(PackageActionResponse {
        package_name: plan.package_name,
        trigger: plan.trigger,
        results: plan.results,
    })
}

pub async fn trigger_target_action<D>(
    deps: &D,
    package_name: &str,
    mock_chroot: &str,
    trigger: BuildTrigger,
    force: bool,
) -> anyhow::Result<PackageActionTargetResult>
where
    D: PackageActionDependencies,
{
    trigger_target_action_inner(deps, package_name, mock_chroot, trigger, force, None).await
}

pub async fn trigger_target_action_for_sync<D>(
    deps: &D,
    package_name: &str,
    mock_chroot: &str,
    trigger: BuildTrigger,
    force: bool,
    sync_operation_id: Uuid,
) -> anyhow::Result<PackageActionTargetResult>
where
    D: PackageActionDependencies,
{
    trigger_target_action_inner(
        deps,
        package_name,
        mock_chroot,
        trigger,
        force,
        Some(sync_operation_id),
    )
    .await
}

async fn trigger_target_action_inner<D>(
    deps: &D,
    package_name: &str,
    mock_chroot: &str,
    trigger: BuildTrigger,
    force: bool,
    sync_operation_id: Option<Uuid>,
) -> anyhow::Result<PackageActionTargetResult>
where
    D: PackageActionDependencies,
{
    let package = deps.get_package_definition(package_name).await?;
    let plan = prepare_package_action(
        deps,
        package,
        Some(mock_chroot),
        trigger,
        force,
        sync_operation_id,
    )
    .await?;
    report_stage(
        deps,
        sync_operation_id,
        SyncStage::EnqueuingBuilds,
        "Reserving and enqueuing build",
    )
    .await?;
    let plan = reserve_and_enqueue_builds(deps, plan).await?;

    plan.results
        .into_iter()
        .find(|result| result.mock_chroot == mock_chroot)
        .ok_or_else(|| {
            anyhow::anyhow!(SynforgeError::NotFound(format!(
                "target {} for package {}",
                mock_chroot, package_name
            )))
        })
}

pub trait PackageActionDependencies:
    PackageDefinitionReader
    + TrackedSourceInspector
    + PackageDefinitionMaterializer
    + PackageDefinitionWriter
    + ActiveTargetBuildReader
    + LastSuccessfulRevisionReader
    + TargetBuildBackoffReader
    + BuildJobWriter
    + BuildQueue
    + SyncRunReporter
    + Send
    + Sync
{
}

impl<T> PackageActionDependencies for T where
    T: PackageDefinitionReader
        + TrackedSourceInspector
        + PackageDefinitionMaterializer
        + PackageDefinitionWriter
        + ActiveTargetBuildReader
        + LastSuccessfulRevisionReader
        + TargetBuildBackoffReader
        + BuildJobWriter
        + BuildQueue
        + SyncRunReporter
        + Send
        + Sync
{
}
