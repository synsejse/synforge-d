use synforge_core::{
    api::BuildJobResponse,
    error::SynforgeError,
    model::{BuildStatus, BuildTrigger},
};
use uuid::Uuid;

use super::{
    ActiveTargetBuildReader, BuildJobReader, BuildQueue, ExistingSourceSyncer,
    PackageDefinitionReader, QueuedBuildRequest, RetryBuildCleaner, RetryJobResetter,
    sync_trigger_from_build_trigger,
};

pub async fn retry_job<D>(deps: &D, job_id: Uuid) -> anyhow::Result<BuildJobResponse>
where
    D: BuildJobReader
        + PackageDefinitionReader
        + ExistingSourceSyncer
        + ActiveTargetBuildReader
        + RetryBuildCleaner
        + RetryJobResetter
        + BuildQueue
        + Send
        + Sync,
{
    let job = deps.get_build_job(job_id).await?;
    if matches!(job.job.status, BuildStatus::Pending | BuildStatus::Running) {
        return Err(anyhow::anyhow!(SynforgeError::Conflict(format!(
            "job {} is still active; use kill before retry",
            job_id
        ))));
    }

    let trigger = BuildTrigger::ManualRebuild;
    let package = deps.get_package_definition(&job.job.package_name).await?;
    let (_, revision) = deps
        .sync_existing_source_tracked(&package, sync_trigger_from_build_trigger(trigger))
        .await?;

    if deps
        .has_active_job_for_target(&job.job.package_name, &job.job.mock_chroot)
        .await?
    {
        return Err(anyhow::anyhow!(SynforgeError::Conflict(
            "retry target is already queued or running".to_string(),
        )));
    }

    deps.cleanup_retry_build(job_id).await?;
    deps.reset_job_for_retry(job_id, trigger, &revision.comparison_key())
        .await?;
    deps.enqueue_build(QueuedBuildRequest {
        package,
        mock_chroot: job.job.mock_chroot.clone(),
        revision,
        trigger,
        job_id,
    })
    .await?;

    deps.get_build_job(job_id).await
}
