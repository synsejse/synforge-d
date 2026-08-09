use std::sync::Arc;

use synforge_core::{
    api::{
        PackageActionDisposition, PackageActionTargetResult, SyncBatchDetailResponse,
        SyncEnqueueResponse,
    },
    error::SynforgeError,
    model::BuildTrigger,
    sync::{SyncOperation, SyncStatus, SyncTriggerType},
};
use synforge_database::{NewSyncRun, SyncRunCompletion, SyncStore};
use tokio::sync::{Semaphore, mpsc, watch};
use tracing::{error, info, warn};
use uuid::Uuid;

use super::SynforgeService;

const MAX_CONCURRENT_SYNCS: usize = 2;

impl SynforgeService {
    pub async fn enqueue_refresh_all_batch(&self) -> anyhow::Result<SyncBatchDetailResponse> {
        let mut package_names: Vec<String> = self
            .registry
            .list_definitions()
            .await?
            .into_iter()
            .filter(|package| package.enabled)
            .map(|package| package.name)
            .collect();
        package_names.sort();
        let batch = self
            .store
            .create_sync_batch(SyncTriggerType::ManualRefresh, package_names.len() as u64)
            .await?;
        let batch_id = Uuid::parse_str(&batch.id)?;
        let mut deduplicated = 0_u64;
        for package_name in package_names {
            match self
                .enqueue_package_sync(
                    &package_name,
                    SyncTriggerType::ManualRefresh,
                    None,
                    Some(batch_id),
                    None,
                )
                .await
            {
                Ok(response) if !response.created => deduplicated += 1,
                Ok(_) => {}
                Err(error) => {
                    warn!(%package_name, %error, "failed to enqueue refresh-all package");
                    self.store
                        .record_sync_batch_enqueue_failure(batch_id, &error.to_string())
                        .await?;
                }
            }
        }
        if deduplicated > 0 {
            self.store
                .record_sync_batch_deduplication(batch_id, deduplicated)
                .await?;
        }
        self.get_sync_batch_detail(batch_id).await
    }

    pub async fn enqueue_package_sync(
        &self,
        package_name: &str,
        trigger_type: SyncTriggerType,
        target_mock_chroot: Option<String>,
        batch_id: Option<Uuid>,
        retry_of: Option<Uuid>,
    ) -> anyhow::Result<SyncEnqueueResponse> {
        let package = self.registry.get_definition(package_name).await?;
        if let Some(target) = target_mock_chroot.as_deref()
            && !package.mock_chroots.iter().any(|value| value == target)
        {
            return Err(anyhow::anyhow!(SynforgeError::NotFound(format!(
                "target {} for package {}",
                target, package_name
            ))));
        }

        let enqueued = self
            .store
            .enqueue_sync_run(NewSyncRun {
                package_name: package_name.to_string(),
                trigger_type,
                target_mock_chroot,
                batch_id,
                retry_of,
            })
            .await?;
        if enqueued.created {
            let id = parse_operation_id(&enqueued.operation)?;
            self.sync_queue_tx
                .send(id)
                .await
                .map_err(|_| anyhow::anyhow!("sync queue is unavailable"))?;
        }
        Ok(SyncEnqueueResponse {
            operation: enqueued.operation,
            created: enqueued.created,
        })
    }

    pub async fn retry_sync_run(&self, id: Uuid) -> anyhow::Result<SyncEnqueueResponse> {
        let operation = self
            .store
            .get_sync_operation(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(id.to_string())))?;
        if !operation.status.is_terminal() {
            return Err(anyhow::anyhow!(SynforgeError::Conflict(
                "only finished sync runs can be retried".to_string()
            )));
        }
        self.enqueue_package_sync(
            &operation.package_name,
            operation.trigger_type,
            operation.target_mock_chroot,
            operation
                .batch_id
                .as_deref()
                .map(Uuid::parse_str)
                .transpose()?,
            Some(id),
        )
        .await
    }

    pub async fn cancel_sync_run(&self, id: Uuid) -> anyhow::Result<SyncOperation> {
        let operation = self
            .store
            .request_sync_cancellation(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(id.to_string())))?;
        if operation.status == SyncStatus::Running && operation.cancellation_requested {
            self.worker_launcher
                .kill_job(id, None, "source sync cancelled")
                .await?;
        }
        if operation.status == SyncStatus::Cancelled
            && let Some(batch_id) = operation.batch_id.as_deref()
        {
            self.store
                .refresh_sync_batch(Uuid::parse_str(batch_id)?)
                .await?;
        }
        Ok(operation)
    }

    pub(super) fn start_sync_queue_runner(
        self: &Arc<Self>,
        mut queue_rx: mpsc::Receiver<Uuid>,
        mut shutdown_rx: watch::Receiver<bool>,
    ) {
        let service = Arc::clone(self);
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_SYNCS));
        let task_tracker = self.task_tracker.clone();
        info!(
            max_concurrent_syncs = MAX_CONCURRENT_SYNCS,
            "starting sync queue runner"
        );
        task_tracker.clone().spawn(async move {
            loop {
                let maybe_id = tokio::select! {
                    _ = shutdown_rx.changed() => break,
                    id = queue_rx.recv() => id,
                };
                let Some(id) = maybe_id else { break };
                let permit = match Arc::clone(&semaphore).acquire_owned().await {
                    Ok(permit) => permit,
                    Err(error) => {
                        error!(%error, "failed to acquire sync semaphore");
                        break;
                    }
                };
                let service = Arc::clone(&service);
                task_tracker.spawn(async move {
                    let _permit = permit;
                    if let Err(error) = service.process_sync_run(id).await {
                        error!(sync_operation_id = %id, %error, "sync processing failed");
                    }
                });
            }
        });
    }

    pub(super) fn recover_queued_sync_runs(self: &Arc<Self>) {
        let service = Arc::clone(self);
        self.task_tracker.spawn(async move {
            match service.store.list_queued_sync_run_ids().await {
                Ok(ids) => {
                    if !ids.is_empty() {
                        info!(count = ids.len(), "recovering queued sync runs");
                    }
                    for id in ids {
                        if service.sync_queue_tx.send(id).await.is_err() {
                            warn!(sync_operation_id = %id, "sync queue closed during recovery");
                            break;
                        }
                    }
                }
                Err(error) => error!(%error, "failed to recover queued sync runs"),
            }
        });
    }

    async fn process_sync_run(&self, id: Uuid) -> anyhow::Result<()> {
        if !self.store.claim_sync_run(id).await? {
            return Ok(());
        }
        let operation = self
            .store
            .get_sync_operation(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("claimed sync operation {} disappeared", id))?;
        info!(
            sync_operation_id = %id,
            package_name = %operation.package_name,
            trigger = ?operation.trigger_type,
            "sync run started"
        );

        let (build_trigger, force) = build_action(operation.trigger_type);
        let deps = self.package_deps();
        let result = match operation.target_mock_chroot.as_deref() {
            Some(target) => self
                .build_service
                .trigger_target_action_for_sync(
                    &deps,
                    &operation.package_name,
                    target,
                    build_trigger,
                    force,
                    id,
                )
                .await
                .map(|result| vec![result]),
            None => self
                .build_service
                .trigger_package_action_for_sync(
                    &deps,
                    &operation.package_name,
                    build_trigger,
                    force,
                    id,
                )
                .await
                .map(|response| response.results),
        };

        let current = self.store.get_sync_operation(id).await?;
        let cancelled = current
            .as_ref()
            .is_some_and(|value| value.cancellation_requested);
        let completion = match result {
            Ok(_) if cancelled => cancelled_completion(),
            Ok(results) => successful_completion(operation.trigger_type, &results),
            Err(_) if cancelled => cancelled_completion(),
            Err(error) => failed_completion(error.to_string()),
        };
        let status = completion.status;
        self.store.finish_sync_run(id, completion).await?;
        if let Some(batch_id) = operation.batch_id.as_deref() {
            self.store
                .refresh_sync_batch(Uuid::parse_str(batch_id)?)
                .await?;
        }
        // Parse workers close their socket before the enclosing sync row is
        // finalized. Publish again after the terminal DB transition to close
        // subscribers that connected inside that narrow race window.
        self.log_broadcaster.publish_complete(id);
        info!(sync_operation_id = %id, ?status, "sync run finished");
        Ok(())
    }
}

fn successful_completion(
    trigger: SyncTriggerType,
    results: &[PackageActionTargetResult],
) -> SyncRunCompletion {
    let count = |disposition| {
        results
            .iter()
            .filter(|result| result.disposition == disposition)
            .count() as u64
    };
    let queued_targets = count(PackageActionDisposition::Queued);
    SyncRunCompletion {
        status: SyncStatus::Succeeded,
        revision: results.iter().find_map(|result| result.revision.clone()),
        previous_revision: None,
        changed: (trigger != SyncTriggerType::ManualRebuild).then_some(queued_targets > 0),
        queued_targets,
        skipped_targets: count(PackageActionDisposition::Skipped),
        blocked_targets: count(PackageActionDisposition::Blocked),
        error_message: None,
    }
}

fn cancelled_completion() -> SyncRunCompletion {
    terminal_completion(SyncStatus::Cancelled, Some("Sync cancelled".to_string()))
}

fn failed_completion(error: String) -> SyncRunCompletion {
    terminal_completion(SyncStatus::Failed, Some(error))
}

fn terminal_completion(status: SyncStatus, error_message: Option<String>) -> SyncRunCompletion {
    SyncRunCompletion {
        status,
        revision: None,
        previous_revision: None,
        changed: None,
        queued_targets: 0,
        skipped_targets: 0,
        blocked_targets: 0,
        error_message,
    }
}

fn build_action(trigger: SyncTriggerType) -> (BuildTrigger, bool) {
    match trigger {
        SyncTriggerType::Poll => (BuildTrigger::Poll, false),
        SyncTriggerType::ManualRefresh => (BuildTrigger::ManualRefresh, false),
        SyncTriggerType::ManualRebuild => (BuildTrigger::ManualRebuild, true),
    }
}

fn parse_operation_id(operation: &SyncOperation) -> anyhow::Result<Uuid> {
    Uuid::parse_str(&operation.id).map_err(Into::into)
}
