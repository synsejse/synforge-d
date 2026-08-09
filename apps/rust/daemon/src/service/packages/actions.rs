use synforge_core::{
    api::{RebuildRequest, RefreshRequest, SyncEnqueueResponse},
    sync::SyncTriggerType,
};
use tracing::info;

use super::SynforgeService;

impl SynforgeService {
    pub async fn trigger_refresh(
        &self,
        package_name: &str,
        _request: RefreshRequest,
    ) -> anyhow::Result<SyncEnqueueResponse> {
        info!(package_name, "manual refresh requested");
        self.enqueue_package_sync(
            package_name,
            SyncTriggerType::ManualRefresh,
            None,
            None,
            None,
        )
        .await
    }

    pub async fn trigger_rebuild(
        &self,
        package_name: &str,
        _request: RebuildRequest,
    ) -> anyhow::Result<SyncEnqueueResponse> {
        info!(package_name, "manual rebuild requested");
        self.enqueue_package_sync(
            package_name,
            SyncTriggerType::ManualRebuild,
            None,
            None,
            None,
        )
        .await
    }

    pub async fn trigger_target_refresh(
        &self,
        package_name: &str,
        mock_chroot: &str,
        _request: RefreshRequest,
    ) -> anyhow::Result<SyncEnqueueResponse> {
        info!(package_name, mock_chroot, "manual target refresh requested");
        self.enqueue_package_sync(
            package_name,
            SyncTriggerType::ManualRefresh,
            Some(mock_chroot.to_string()),
            None,
            None,
        )
        .await
    }

    pub async fn trigger_target_rebuild(
        &self,
        package_name: &str,
        mock_chroot: &str,
        _request: RebuildRequest,
    ) -> anyhow::Result<SyncEnqueueResponse> {
        info!(package_name, mock_chroot, "manual target rebuild requested");
        self.enqueue_package_sync(
            package_name,
            SyncTriggerType::ManualRebuild,
            Some(mock_chroot.to_string()),
            None,
            None,
        )
        .await
    }
}
