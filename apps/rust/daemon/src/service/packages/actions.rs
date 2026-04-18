use synforge_core::{
    api::{PackageActionResponse, PackageActionTargetResult, RebuildRequest, RefreshRequest},
    model::BuildTrigger,
};
use tracing::info;

use super::{SynforgeService, deps::log_action_response};

impl SynforgeService {
    pub async fn trigger_refresh(
        &self,
        package_name: &str,
        _request: RefreshRequest,
    ) -> anyhow::Result<PackageActionResponse> {
        info!(package_name, trigger = ?BuildTrigger::ManualRefresh, "manual refresh requested");
        let response = self
            .build_service
            .trigger_package_action(
                &self.package_deps(),
                package_name,
                BuildTrigger::ManualRefresh,
                false,
            )
            .await?;
        log_action_response(package_name, BuildTrigger::ManualRefresh, &response.results);
        Ok(response)
    }

    pub async fn trigger_rebuild(
        &self,
        package_name: &str,
        _request: RebuildRequest,
    ) -> anyhow::Result<PackageActionResponse> {
        info!(package_name, trigger = ?BuildTrigger::ManualRebuild, "manual rebuild requested");
        let response = self
            .build_service
            .trigger_package_action(
                &self.package_deps(),
                package_name,
                BuildTrigger::ManualRebuild,
                true,
            )
            .await?;
        log_action_response(package_name, BuildTrigger::ManualRebuild, &response.results);
        Ok(response)
    }

    pub async fn trigger_target_refresh(
        &self,
        package_name: &str,
        mock_chroot: &str,
        _request: RefreshRequest,
    ) -> anyhow::Result<PackageActionTargetResult> {
        info!(
            package_name,
            mock_chroot,
            trigger = ?BuildTrigger::ManualRefresh,
            "manual target refresh requested"
        );
        let result = self
            .build_service
            .trigger_target_action(
                &self.package_deps(),
                package_name,
                mock_chroot,
                BuildTrigger::ManualRefresh,
                false,
            )
            .await?;
        info!(
            package_name,
            mock_chroot,
            trigger = ?BuildTrigger::ManualRefresh,
            disposition = ?result.disposition,
            "manual target refresh scheduled"
        );
        Ok(result)
    }

    pub async fn trigger_target_rebuild(
        &self,
        package_name: &str,
        mock_chroot: &str,
        _request: RebuildRequest,
    ) -> anyhow::Result<PackageActionTargetResult> {
        info!(
            package_name,
            mock_chroot,
            trigger = ?BuildTrigger::ManualRebuild,
            "manual target rebuild requested"
        );
        let result = self
            .build_service
            .trigger_target_action(
                &self.package_deps(),
                package_name,
                mock_chroot,
                BuildTrigger::ManualRebuild,
                true,
            )
            .await?;
        info!(
            package_name,
            mock_chroot,
            trigger = ?BuildTrigger::ManualRebuild,
            disposition = ?result.disposition,
            "manual target rebuild scheduled"
        );
        Ok(result)
    }
}
