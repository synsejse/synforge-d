use synforge_core::{
    api::BuildJobResponse,
    error::SynforgeError,
    model::{BuildStatus, BuildTrigger},
};
use uuid::Uuid;

use super::{
    ActiveTargetBuildReader, BuildJobReader, BuildQueue, ExistingSourceSyncer,
    PackageDefinitionReader, QueuedBuildRequest, RetryBuildCleaner, RetryJobResetter,
    RetryPublishedFilesReader, sync_trigger_from_build_trigger,
};

pub async fn retry_job<D>(deps: &D, job_id: Uuid) -> anyhow::Result<BuildJobResponse>
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

    let published_files = deps.get_retry_published_files(job_id).await?;
    if !deps
        .reset_job_for_retry(job_id, trigger, &revision.comparison_key())
        .await?
    {
        return Err(anyhow::anyhow!(SynforgeError::Conflict(
            "retry target is already queued or running".to_string(),
        )));
    }

    if let Err(error) = deps.cleanup_retry_build(job_id, &published_files).await {
        let message = format!("failed to clean reserved retry: {error}");
        deps.cancel_job_retry(job_id, &message).await?;
        return Err(error);
    }
    let queued = QueuedBuildRequest {
        package,
        mock_chroot: job.job.mock_chroot.clone(),
        revision,
        trigger,
        job_id,
    };
    if let Err(error) = deps.enqueue_build(queued).await {
        let message = format!("failed to enqueue reserved retry: {error}");
        deps.cancel_job_retry(job_id, &message).await?;
        return Err(error);
    }

    deps.get_build_job(job_id).await
}
