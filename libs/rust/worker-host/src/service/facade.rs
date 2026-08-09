//! Facade for build and worker workflows.

use synforge_core::{
    api::{BuildJobResponse, PackageActionResponse, PackageActionTargetResult},
    model::BuildTrigger,
};
use synforge_git_sync::{PackageDefinitionMaterializer, PackageDefinitionWriter};

use super::commands::SyncRunReporter;
use super::commands::{
    ActiveTargetBuildReader, BuildJobReader, BuildJobWriter, BuildQueue, ExistingSourceSyncer,
    LastSuccessfulRevisionReader, PackageDefinitionReader, RetryBuildCleaner, RetryJobResetter,
    RetryPublishedFilesReader, TargetBuildBackoffReader, TrackedSourceInspector, retry_job,
    trigger_package_action, trigger_package_action_for_sync, trigger_target_action,
    trigger_target_action_for_sync,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct BuildService;

impl BuildService {
    pub async fn trigger_package_action<D>(
        &self,
        deps: &D,
        package_name: &str,
        trigger: BuildTrigger,
        force: bool,
    ) -> anyhow::Result<PackageActionResponse>
    where
        D: PackageDefinitionReader
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
            + Sync,
    {
        trigger_package_action(deps, package_name, trigger, force).await
    }

    pub async fn trigger_package_action_for_sync<D>(
        &self,
        deps: &D,
        package_name: &str,
        trigger: BuildTrigger,
        force: bool,
        sync_operation_id: uuid::Uuid,
    ) -> anyhow::Result<PackageActionResponse>
    where
        D: PackageDefinitionReader
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
            + Sync,
    {
        trigger_package_action_for_sync(deps, package_name, trigger, force, sync_operation_id).await
    }

    pub async fn trigger_target_action<D>(
        &self,
        deps: &D,
        package_name: &str,
        mock_chroot: &str,
        trigger: BuildTrigger,
        force: bool,
    ) -> anyhow::Result<PackageActionTargetResult>
    where
        D: PackageDefinitionReader
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
            + Sync,
    {
        trigger_target_action(deps, package_name, mock_chroot, trigger, force).await
    }

    pub async fn trigger_target_action_for_sync<D>(
        &self,
        deps: &D,
        package_name: &str,
        mock_chroot: &str,
        trigger: BuildTrigger,
        force: bool,
        sync_operation_id: uuid::Uuid,
    ) -> anyhow::Result<PackageActionTargetResult>
    where
        D: PackageDefinitionReader
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
            + Sync,
    {
        trigger_target_action_for_sync(
            deps,
            package_name,
            mock_chroot,
            trigger,
            force,
            sync_operation_id,
        )
        .await
    }

    pub async fn retry_job<D>(
        &self,
        deps: &D,
        job_id: uuid::Uuid,
    ) -> anyhow::Result<BuildJobResponse>
    where
        D: BuildJobReader
            + PackageDefinitionReader
            + ExistingSourceSyncer
            + ActiveTargetBuildReader
            + RetryPublishedFilesReader
            + RetryBuildCleaner
            + RetryJobResetter
            + BuildQueue
            + Send
            + Sync,
    {
        retry_job(deps, job_id).await
    }
}
