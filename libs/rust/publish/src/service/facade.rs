//! Facade for repository workflows.

use std::path::{Path, PathBuf};

use synforge_core::{
    api::{
        ExportRepoSigningKeyResponse, ExportRepoSigningPublicKeyResponse,
        GenerateRepoSigningKeyResponse, ImportRepoSigningKeyRequest, ImportRepoSigningKeyResponse,
        RepoInventoryResponse, RepoSigningReconcileProgressResponse, RepoSigningStatusResponse,
        RepoSummaryResponse, TestRepoSigningResponse, UpdateRepoSigningConfigRequest,
    },
    model::ArtifactKind,
};

use super::{
    commands::{
        RepoArtifactCatalog, RepoSigningCommandRunner, RepoSigningProgressWriter,
        RepoSigningSettingsWriter, export_repo_signing_private_key, export_repo_signing_public_key,
        generate_repo_signing_key, import_repo_signing_key, remove_repo_signing_key,
        test_repo_signing, update_repo_signing_config,
    },
    queries::{
        RepoInventoryReader, RepoSigningConfigLoader, RepoSigningInspector,
        RepoSigningProgressReader, RepoSummaryReader, get_repo_inventory,
        get_repo_signing_reconcile_progress, get_repo_signing_status, get_repo_summary,
        resolve_repo_file_path,
    },
};

#[derive(Debug, Clone, Default)]
pub struct RepoService;

impl RepoService {
    pub async fn get_repo_inventory<D>(
        &self,
        deps: &D,
        limit: Option<usize>,
        offset: Option<usize>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<RepoInventoryResponse>
    where
        D: RepoInventoryReader + Send + Sync,
    {
        get_repo_inventory(deps, limit, offset, package_name, mock_chroot, kind).await
    }

    pub async fn get_repo_summary<D>(&self, deps: &D) -> anyhow::Result<RepoSummaryResponse>
    where
        D: RepoSummaryReader + Send + Sync,
    {
        get_repo_summary(deps).await
    }

    pub async fn resolve_repo_file_path(
        &self,
        repo_root: &Path,
        relative_repo_path: &str,
    ) -> anyhow::Result<PathBuf> {
        resolve_repo_file_path(repo_root, relative_repo_path).await
    }

    pub async fn get_repo_signing_status<D>(
        &self,
        deps: &D,
        can_export_private_key: bool,
    ) -> anyhow::Result<RepoSigningStatusResponse>
    where
        D: RepoSigningConfigLoader + RepoSigningInspector + Send + Sync,
    {
        get_repo_signing_status(deps, can_export_private_key).await
    }

    pub async fn get_repo_signing_reconcile_progress<D>(
        &self,
        deps: &D,
    ) -> anyhow::Result<RepoSigningReconcileProgressResponse>
    where
        D: RepoSigningProgressReader + Send + Sync,
    {
        get_repo_signing_reconcile_progress(deps).await
    }

    pub async fn update_repo_signing_config<D>(
        &self,
        deps: &D,
        request: UpdateRepoSigningConfigRequest,
        can_export_private_key: bool,
    ) -> anyhow::Result<RepoSigningStatusResponse>
    where
        D: RepoSigningConfigLoader
            + RepoSigningInspector
            + RepoSigningSettingsWriter
            + RepoSigningCommandRunner
            + RepoArtifactCatalog
            + RepoSigningProgressWriter
            + Send
            + Sync,
    {
        update_repo_signing_config(deps, request, can_export_private_key).await
    }

    pub async fn generate_repo_signing_key<D>(
        &self,
        deps: &D,
        can_export_private_key: bool,
    ) -> anyhow::Result<GenerateRepoSigningKeyResponse>
    where
        D: RepoSigningConfigLoader
            + RepoSigningInspector
            + RepoSigningSettingsWriter
            + RepoSigningCommandRunner
            + Send
            + Sync,
    {
        generate_repo_signing_key(deps, can_export_private_key).await
    }

    pub async fn import_repo_signing_key<D>(
        &self,
        deps: &D,
        request: ImportRepoSigningKeyRequest,
        can_export_private_key: bool,
    ) -> anyhow::Result<ImportRepoSigningKeyResponse>
    where
        D: RepoSigningConfigLoader
            + RepoSigningInspector
            + RepoSigningSettingsWriter
            + RepoSigningCommandRunner
            + Send
            + Sync,
    {
        import_repo_signing_key(deps, request, can_export_private_key).await
    }

    pub async fn remove_repo_signing_key<D>(
        &self,
        deps: &D,
        can_export_private_key: bool,
    ) -> anyhow::Result<RepoSigningStatusResponse>
    where
        D: RepoSigningConfigLoader
            + RepoSigningInspector
            + RepoSigningSettingsWriter
            + RepoSigningCommandRunner
            + RepoArtifactCatalog
            + Send
            + Sync,
    {
        remove_repo_signing_key(deps, can_export_private_key).await
    }

    pub async fn test_repo_signing<D>(&self, deps: &D) -> anyhow::Result<TestRepoSigningResponse>
    where
        D: RepoSigningConfigLoader
            + RepoSigningInspector
            + RepoSigningCommandRunner
            + RepoArtifactCatalog
            + Send
            + Sync,
    {
        test_repo_signing(deps).await
    }

    pub async fn export_repo_signing_private_key<D>(
        &self,
        deps: &D,
        authorized: bool,
    ) -> anyhow::Result<ExportRepoSigningKeyResponse>
    where
        D: RepoSigningConfigLoader
            + RepoSigningInspector
            + RepoSigningSettingsWriter
            + RepoSigningCommandRunner
            + Send
            + Sync,
    {
        export_repo_signing_private_key(deps, authorized).await
    }

    pub async fn export_repo_signing_public_key<D>(
        &self,
        deps: &D,
    ) -> anyhow::Result<ExportRepoSigningPublicKeyResponse>
    where
        D: RepoSigningConfigLoader + RepoSigningCommandRunner + Send + Sync,
    {
        export_repo_signing_public_key(deps).await
    }
}
