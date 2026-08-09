use synforge_core::{
    api::{PackageActionDisposition, PackageActionTargetResult},
    error::SynforgeError,
    model::{BuildJob, BuildStatus, BuildTrigger, now_utc},
    package::{PackageDefinition, parse_mock_chroot},
    sync::SyncStage,
    validation::{PackageDefinitionValidator, Validator},
};
use synforge_git_sync::{
    PackageDefinitionMaterializer, PackageDefinitionWriter, PackageMaterializationOptions,
};
use uuid::Uuid;

use super::{
    ActiveTargetBuildReader, BuildJobWriter, BuildQueue, LastSuccessfulRevisionReader,
    QueuedBuildRequest, SyncRunReporter, TargetBuildBackoffReader, TrackedSourceInspector,
    sync_trigger_from_build_trigger,
};

mod entry;

pub use entry::{
    trigger_package_action, trigger_package_action_for_sync, trigger_target_action,
    trigger_target_action_for_sync,
};

struct PackageActionPlan {
    package_name: String,
    trigger: BuildTrigger,
    builds: Vec<PlannedBuild>,
    results: Vec<PackageActionTargetResult>,
}

struct PlannedBuild {
    job: BuildJob,
    queued: QueuedBuildRequest,
}

async fn reserve_and_enqueue_builds<D>(
    deps: &D,
    mut plan: PackageActionPlan,
) -> anyhow::Result<PackageActionPlan>
where
    D: BuildJobWriter + BuildQueue + Send + Sync,
{
    for planned in std::mem::take(&mut plan.builds) {
        let revision = planned.job.revision.clone();
        if !deps.insert_build_job(&planned.job).await? {
            plan.results.push(PackageActionTargetResult {
                package_name: planned.job.package_name,
                mock_chroot: planned.job.mock_chroot,
                disposition: PackageActionDisposition::Blocked,
                reason: Some("pending_or_running".to_string()),
                job_id: None,
                revision: Some(revision),
            });
            continue;
        }

        if let Err(error) = deps.enqueue_build(planned.queued).await {
            let message = format!("failed to enqueue reserved build: {error}");
            deps.cancel_build_job(planned.job.id, &message).await?;
            return Err(error);
        }
        plan.results.push(PackageActionTargetResult {
            package_name: planned.job.package_name,
            mock_chroot: planned.job.mock_chroot,
            disposition: PackageActionDisposition::Queued,
            reason: None,
            job_id: Some(planned.job.id),
            revision: Some(revision),
        });
    }
    Ok(plan)
}

async fn prepare_package_action<D>(
    deps: &D,
    package: PackageDefinition,
    target_mock_chroot: Option<&str>,
    trigger: BuildTrigger,
    force: bool,
    sync_operation_id: Option<Uuid>,
) -> anyhow::Result<PackageActionPlan>
where
    D: TrackedSourceInspector
        + PackageDefinitionMaterializer
        + PackageDefinitionWriter
        + ActiveTargetBuildReader
        + LastSuccessfulRevisionReader
        + TargetBuildBackoffReader
        + SyncRunReporter
        + Send
        + Sync,
{
    PackageDefinitionValidator.validate(&package)?;
    let inspected = deps
        .inspect_source_tracked(
            &package.name,
            &package.source,
            package.build_timeout_seconds,
            sync_trigger_from_build_trigger(trigger),
            sync_operation_id,
        )
        .await?;
    let revision_key = inspected.revision.comparison_key();

    report_stage(
        deps,
        sync_operation_id,
        SyncStage::PlanningBuilds,
        "Comparing source revision with build history",
    )
    .await?;

    let build_chroots = match target_mock_chroot {
        Some(mock_chroot) => {
            if package
                .mock_chroots
                .iter()
                .any(|value| value == mock_chroot)
            {
                vec![mock_chroot.to_string()]
            } else {
                return Err(anyhow::anyhow!(SynforgeError::NotFound(format!(
                    "target {} for package {}",
                    mock_chroot, package.name
                ))));
            }
        }
        None => package.mock_chroots.clone(),
    };

    let mut queued_chroots = Vec::new();
    let mut results = Vec::new();
    for mock_chroot in &build_chroots {
        if deps
            .has_active_job_for_target(&package.name, mock_chroot)
            .await?
        {
            results.push(PackageActionTargetResult {
                package_name: package.name.clone(),
                mock_chroot: mock_chroot.clone(),
                disposition: PackageActionDisposition::Blocked,
                reason: Some("pending_or_running".to_string()),
                job_id: None,
                revision: Some(revision_key.clone()),
            });
            continue;
        }
        if trigger == BuildTrigger::Poll
            && let Some(wait_seconds) = deps
                .get_target_backoff_wait_seconds(&package.name, mock_chroot)
                .await?
        {
            results.push(PackageActionTargetResult {
                package_name: package.name.clone(),
                mock_chroot: mock_chroot.clone(),
                disposition: PackageActionDisposition::Skipped,
                reason: Some(format!("backoff:{}s", wait_seconds)),
                job_id: None,
                revision: Some(revision_key.clone()),
            });
            continue;
        }
        let previous_revision = deps
            .get_last_successful_revision(&package.name, mock_chroot)
            .await?;
        if force || previous_revision.as_deref() != Some(revision_key.as_str()) {
            queued_chroots.push(mock_chroot.clone());
        } else {
            results.push(PackageActionTargetResult {
                package_name: package.name.clone(),
                mock_chroot: mock_chroot.clone(),
                disposition: PackageActionDisposition::Skipped,
                reason: Some("no_source_change".to_string()),
                job_id: None,
                revision: Some(revision_key.clone()),
            });
        }
    }

    let updated_package = deps
        .materialize_package_definition(
            &package.source,
            &inspected,
            PackageMaterializationOptions {
                enabled: package.enabled,
                publish_srpm: package.publish_srpm,
                publish_debuginfo: package.publish_debuginfo,
                network_access: package.network_access,
                ccache_enabled: package.ccache_enabled,
                ccache_max_size_mb: package.ccache_max_size_mb,
                mock_chroots: package.mock_chroots.clone(),
                poll_interval_seconds: package.poll_interval_seconds,
                build_timeout_seconds: package.build_timeout_seconds,
                package_history_count: package.package_history_count,
                cpu_limit_millicores: package.cpu_limit_millicores,
                memory_limit_mb: package.memory_limit_mb,
                build_env: package.build_env.clone(),
            },
        )
        .await?;
    report_stage(
        deps,
        sync_operation_id,
        SyncStage::UpdatingPackage,
        "Updating package metadata",
    )
    .await?;
    deps.upsert_package_definition(&updated_package).await?;

    let now = now_utc();
    let mut builds = Vec::new();
    for mock_chroot in queued_chroots {
        parse_mock_chroot(&mock_chroot)
            .ok_or_else(|| anyhow::anyhow!("invalid mock chroot {}", mock_chroot))?;
        let job_id = Uuid::now_v7();
        let job = BuildJob {
            id: job_id,
            package_name: updated_package.name.clone(),
            mock_chroot: mock_chroot.clone(),
            revision: revision_key.clone(),
            trigger,
            status: BuildStatus::Pending,
            sync_operation_id,
            spec_file: updated_package.spec_file.clone(),
            worker_container_id: None,
            created_at: now,
            updated_at: now,
            started_at: None,
            finished_at: None,
            signed_at: None,
            error_message: None,
            deleted_at: None,
        };
        let queued = QueuedBuildRequest {
            package: updated_package.clone(),
            mock_chroot,
            revision: inspected.revision.clone(),
            trigger,
            job_id,
        };
        builds.push(PlannedBuild { job, queued });
    }

    Ok(PackageActionPlan {
        package_name: updated_package.name,
        trigger,
        builds,
        results,
    })
}

async fn report_stage<D>(
    deps: &D,
    operation_id: Option<Uuid>,
    stage: SyncStage,
    message: &str,
) -> anyhow::Result<()>
where
    D: SyncRunReporter + Send + Sync,
{
    if let Some(operation_id) = operation_id
        && !deps.advance_sync_run(operation_id, stage, message).await?
    {
        anyhow::bail!("sync operation was cancelled or already finalized");
    }
    Ok(())
}

#[cfg(test)]
#[path = "package_actions_tests.rs"]
mod tests;
