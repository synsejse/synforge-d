use std::path::Path;

use async_trait::async_trait;
use synforge_core::{
    api::RepoSigningReconcileProgressView,
    config::DaemonConfig,
    model::{ArtifactSignature, PublishedRepoFile},
};
use uuid::Uuid;

use super::state::{RepoSigningKeyIdentity, RepoSigningSettingsUpdate};

mod signing_keys;
mod signing_reconcile;

pub use signing_keys::{
    export_repo_signing_private_key, export_repo_signing_public_key, generate_repo_signing_key,
    import_repo_signing_key, remove_repo_signing_key, test_repo_signing,
    update_repo_signing_config,
};

#[async_trait]
pub trait RepoSigningSettingsWriter {
    async fn persist_repo_signing_settings(
        &self,
        update: RepoSigningSettingsUpdate,
    ) -> anyhow::Result<()>;
}

#[async_trait]
pub trait RepoSigningCommandRunner {
    async fn generate_new_managed_signing_key(
        &self,
        config: &DaemonConfig,
    ) -> anyhow::Result<RepoSigningKeyIdentity>;

    async fn import_private_key(
        &self,
        config: &DaemonConfig,
        armored_private_key: &str,
    ) -> anyhow::Result<RepoSigningKeyIdentity>;

    async fn remove_all_keys(&self, config: &DaemonConfig) -> anyhow::Result<()>;

    async fn export_private_key(
        &self,
        config: &DaemonConfig,
        key_id: &str,
    ) -> anyhow::Result<String>;

    async fn resolve_configured_signing_identity(
        &self,
        config: &DaemonConfig,
    ) -> anyhow::Result<RepoSigningKeyIdentity>;

    async fn export_public_key(
        &self,
        config: &DaemonConfig,
        key_id: &str,
    ) -> anyhow::Result<String>;

    async fn reconcile_repo_metadata_signature(&self, config: &DaemonConfig) -> anyhow::Result<()>;

    async fn add_signature_to_artifact(
        &self,
        config: &DaemonConfig,
        key_id: &str,
        artifact_path: &Path,
    ) -> anyhow::Result<()>;

    async fn remove_signature_from_artifact(&self, artifact_path: &Path) -> anyhow::Result<()>;

    async fn recompute_artifact_metadata(
        &self,
        artifact_path: &Path,
    ) -> anyhow::Result<(String, u64)>;
}

#[async_trait]
pub trait RepoArtifactCatalog {
    async fn count_all_published_repo_files(&self) -> anyhow::Result<u64>;

    async fn list_all_published_repo_files(
        &self,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<Vec<PublishedRepoFile>>;

    async fn update_build_artifact_metadata(
        &self,
        artifact_id: Uuid,
        sha256: String,
        size_bytes: u64,
    ) -> anyhow::Result<()>;

    async fn upsert_artifact_signatures(
        &self,
        signatures: Vec<ArtifactSignature>,
    ) -> anyhow::Result<()>;
}

#[async_trait]
pub trait RepoSigningProgressWriter {
    async fn save_repo_signing_reconcile_progress(
        &self,
        progress: RepoSigningReconcileProgressView,
    );
}
