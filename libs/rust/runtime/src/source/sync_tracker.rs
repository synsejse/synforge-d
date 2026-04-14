use synforge_core::model::BuildTrigger;
use synforge_core::sync::{SyncStatus, SyncTriggerType};
use tracing::warn;

use synforge_store::{DieselStore, SyncStore};

#[derive(Clone)]
pub struct SyncStatusTracker {
    store: DieselStore,
}

impl SyncStatusTracker {
    pub fn new(store: DieselStore) -> Self {
        Self { store }
    }

    pub async fn record_sync(
        &self,
        package_name: &str,
        trigger: SyncTriggerType,
        result: &SyncResult,
    ) -> anyhow::Result<()> {
        match result {
            SyncResult::Success { revision } => {
                self.store
                    .insert_sync_operation(
                        package_name,
                        trigger,
                        SyncStatus::Succeeded,
                        Some(revision),
                        None,
                    )
                    .await
            }
            SyncResult::Failure { error } => {
                self.store
                    .insert_sync_operation(
                        package_name,
                        trigger,
                        SyncStatus::Failed,
                        None,
                        Some(error),
                    )
                    .await
            }
        }
    }

    pub async fn record_sync_safe(
        &self,
        package_name: &str,
        trigger: SyncTriggerType,
        result: &SyncResult,
    ) {
        if let Err(error) = self.record_sync(package_name, trigger, result).await {
            warn!(
                package_name = %package_name,
                trigger = ?trigger,
                error = %error,
                "failed to record sync operation"
            );
        }
    }
}

#[derive(Debug, Clone)]
pub enum SyncResult {
    Success { revision: String },
    Failure { error: String },
}

impl From<Result<String, anyhow::Error>> for SyncResult {
    fn from(result: Result<String, anyhow::Error>) -> Self {
        match result {
            Ok(revision) => SyncResult::Success { revision },
            Err(error) => SyncResult::Failure {
                error: error.to_string(),
            },
        }
    }
}

pub fn sync_trigger_from_build_trigger(trigger: &BuildTrigger) -> SyncTriggerType {
    match trigger {
        BuildTrigger::Poll => SyncTriggerType::Poll,
        BuildTrigger::ManualRefresh => SyncTriggerType::ManualRefresh,
        BuildTrigger::ManualRebuild => SyncTriggerType::ManualRebuild,
        BuildTrigger::Api => SyncTriggerType::ManualRebuild,
    }
}
