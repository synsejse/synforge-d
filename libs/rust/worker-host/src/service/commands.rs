//! Command-side operations for build orchestration.

use async_trait::async_trait;
use synforge_core::{
    api::BuildJobResponse,
    model::{BuildJob, BuildTrigger, PublishedRepoFile},
    package::{PackageDefinition, SpecRevision},
    sync::SyncTriggerType,
};
use synforge_git_sync::InspectedPackageSource;
use uuid::Uuid;

use super::state::QueuedBuildRequest;

mod package_actions;
mod retry;

pub use package_actions::{trigger_package_action, trigger_target_action};
pub use retry::retry_job;

#[async_trait]
pub trait PackageDefinitionReader {
    async fn get_package_definition(&self, package_name: &str)
    -> anyhow::Result<PackageDefinition>;
}

#[async_trait]
pub trait PackageDefinitionCatalog {
    async fn list_package_definitions(&self) -> anyhow::Result<Vec<PackageDefinition>>;
}

#[async_trait]
pub trait TrackedSourceInspector {
    async fn inspect_source_tracked(
        &self,
        package_name: &str,
        source: &synforge_core::package::SpecSource,
        timeout_seconds: u64,
        trigger: SyncTriggerType,
    ) -> anyhow::Result<InspectedPackageSource>;
}

#[async_trait]
pub trait ActiveTargetBuildReader {
    async fn has_active_job_for_target(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<bool>;
}

#[async_trait]
pub trait LastSuccessfulRevisionReader {
    async fn get_last_successful_revision(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<Option<String>>;
}

#[async_trait]
pub trait TargetBuildBackoffReader {
    async fn get_target_backoff_wait_seconds(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<Option<u64>>;
}

#[async_trait]
pub trait BuildJobWriter {
    async fn insert_build_job(&self, job: &BuildJob) -> anyhow::Result<bool>;
    async fn cancel_build_job(&self, job_id: Uuid, message: &str) -> anyhow::Result<()>;
}

#[async_trait]
pub trait BuildQueue {
    async fn enqueue_build(&self, build: QueuedBuildRequest) -> anyhow::Result<()>;
}

#[async_trait]
pub trait BuildJobReader {
    async fn get_build_job(&self, job_id: Uuid) -> anyhow::Result<BuildJobResponse>;
}

#[async_trait]
pub trait ExistingSourceSyncer {
    async fn sync_existing_source_tracked(
        &self,
        package: &PackageDefinition,
        trigger: SyncTriggerType,
    ) -> anyhow::Result<(PackageDefinition, SpecRevision)>;
}

#[async_trait]
pub trait RetryBuildCleaner {
    async fn cleanup_retry_build(
        &self,
        job_id: Uuid,
        published_files: &[PublishedRepoFile],
    ) -> anyhow::Result<()>;
}

#[async_trait]
pub trait RetryPublishedFilesReader {
    async fn get_retry_published_files(
        &self,
        job_id: Uuid,
    ) -> anyhow::Result<Vec<PublishedRepoFile>>;
}

#[async_trait]
pub trait RetryJobResetter {
    async fn reset_job_for_retry(
        &self,
        job_id: Uuid,
        trigger: BuildTrigger,
        revision: &str,
    ) -> anyhow::Result<bool>;
    async fn cancel_job_retry(&self, job_id: Uuid, message: &str) -> anyhow::Result<()>;
}

pub(super) fn sync_trigger_from_build_trigger(trigger: BuildTrigger) -> SyncTriggerType {
    match trigger {
        BuildTrigger::Poll => SyncTriggerType::Poll,
        BuildTrigger::ManualRefresh => SyncTriggerType::ManualRefresh,
        BuildTrigger::ManualRebuild | BuildTrigger::Api => SyncTriggerType::ManualRebuild,
    }
}
