#[path = "repo/config.rs"]
mod config;
#[path = "repo/deps.rs"]
mod deps;

use std::path::PathBuf;

use super::SynforgeService;
use synforge_core::{
    api::{
        ExportRepoSigningKeyResponse, ExportRepoSigningPublicKeyResponse,
        GenerateRepoSigningKeyResponse, ImportRepoSigningKeyRequest, ImportRepoSigningKeyResponse,
        RepoInventoryResponse, RepoSigningReconcileProgressResponse, RepoSigningStatusResponse,
        RepoSummaryResponse, TestRepoSigningResponse, UpdateRepoSigningConfigRequest,
    },
    model::{ArtifactKind, UserAccount},
};
use uuid::Uuid;

impl SynforgeService {
    pub async fn get_repo_inventory(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<RepoInventoryResponse> {
        self.repo_service
            .get_repo_inventory(
                &self.repo_store(),
                limit,
                offset,
                package_name,
                mock_chroot,
                kind,
            )
            .await
    }

    pub async fn get_repo_summary(&self) -> anyhow::Result<RepoSummaryResponse> {
        self.repo_service.get_repo_summary(&self.repo_store()).await
    }

    pub async fn resolve_repo_file_path(
        &self,
        relative_repo_path: &str,
    ) -> anyhow::Result<PathBuf> {
        self.repo_service
            .resolve_repo_file_path(
                self.config.runtime_paths().repo_dir(),
                relative_repo_path,
            )
            .await
    }

    pub async fn get_repo_signing_status(
        &self,
        current_user: &UserAccount,
    ) -> anyhow::Result<RepoSigningStatusResponse> {
        let deps = self.repo_signing_deps();
        self.repo_service
            .get_repo_signing_status(&deps, self.is_bootstrap_admin_user(current_user.id).await?)
            .await
    }

    pub async fn get_repo_signing_reconcile_progress(
        &self,
    ) -> anyhow::Result<RepoSigningReconcileProgressResponse> {
        let deps = self.repo_signing_deps();
        self.repo_service
            .get_repo_signing_reconcile_progress(&deps)
            .await
    }

    pub async fn update_repo_signing_config(
        &self,
        request: UpdateRepoSigningConfigRequest,
        current_user_id: Uuid,
    ) -> anyhow::Result<RepoSigningStatusResponse> {
        let deps = self.repo_signing_deps();
        self.repo_service
            .update_repo_signing_config(
                &deps,
                request,
                self.is_bootstrap_admin_user(current_user_id).await?,
            )
            .await
    }

    pub async fn generate_repo_signing_key(
        &self,
        current_user_id: Uuid,
    ) -> anyhow::Result<GenerateRepoSigningKeyResponse> {
        let deps = self.repo_signing_deps();
        self.repo_service
            .generate_repo_signing_key(&deps, self.is_bootstrap_admin_user(current_user_id).await?)
            .await
    }

    pub async fn import_repo_signing_key(
        &self,
        request: ImportRepoSigningKeyRequest,
        current_user_id: Uuid,
    ) -> anyhow::Result<ImportRepoSigningKeyResponse> {
        let deps = self.repo_signing_deps();
        self.repo_service
            .import_repo_signing_key(
                &deps,
                request,
                self.is_bootstrap_admin_user(current_user_id).await?,
            )
            .await
    }

    pub async fn remove_repo_signing_key(
        &self,
        current_user_id: Uuid,
    ) -> anyhow::Result<RepoSigningStatusResponse> {
        let deps = self.repo_signing_deps();
        self.repo_service
            .remove_repo_signing_key(&deps, self.is_bootstrap_admin_user(current_user_id).await?)
            .await
    }

    pub async fn test_repo_signing(&self) -> anyhow::Result<TestRepoSigningResponse> {
        let deps = self.repo_signing_deps();
        self.repo_service.test_repo_signing(&deps).await
    }

    pub async fn export_repo_signing_private_key(
        &self,
        current_user: &UserAccount,
    ) -> anyhow::Result<ExportRepoSigningKeyResponse> {
        let deps = self.repo_signing_deps();
        self.repo_service
            .export_repo_signing_private_key(
                &deps,
                self.is_bootstrap_admin_user(current_user.id).await?,
            )
            .await
    }

    pub async fn export_repo_signing_public_key(
        &self,
    ) -> anyhow::Result<ExportRepoSigningPublicKeyResponse> {
        let deps = self.repo_signing_deps();
        self.repo_service
            .export_repo_signing_public_key(&deps)
            .await
    }
}
