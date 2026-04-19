use std::{collections::BTreeMap, path::Path};

use async_trait::async_trait;
use synforge_core::{
    api::RepoSigningStatusView,
    config::{DaemonConfig, RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED},
    model::{ArtifactSignature, PublishedRepoFile},
};
use synforge_database::{
    DieselStore, repo::PostgresRepoStore, runtime_settings::PostgresRuntimeSettingsStore,
};
use synforge_publish::{
    RepoArtifactCatalog, RepoFileStorage, RepoSigningCommandRunner, RepoSigningConfigLoader,
    RepoSigningInspector, RepoSigningKeyIdentity, RepoSigningProgressReader,
    RepoSigningProgressWriter, RepoSigningSettingsUpdate, RepoSigningSettingsWriter,
    RuntimeRepoAdapter,
};
use synforge_state::SigningReconcileProgressState;
use uuid::Uuid;

use crate::service::{SynforgeService, config::load_effective_daemon_config_from_store};

use super::config::sync_keyring_from_runtime_settings;

#[derive(Clone)]
pub(crate) struct RepoSigningDeps {
    store: DieselStore,
    base_config: DaemonConfig,
    signing_progress: SigningReconcileProgressState,
}

impl RepoSigningDeps {
    fn repo_adapter(&self) -> RuntimeRepoAdapter {
        RuntimeRepoAdapter::new()
    }

    fn repo_store(&self) -> PostgresRepoStore {
        PostgresRepoStore::new(self.store.clone())
    }
}

#[async_trait]
impl RepoFileStorage for RepoSigningDeps {
    async fn restore_repo_file(
        &self,
        relative_repo_path: &Path,
        destination_path: &Path,
    ) -> anyhow::Result<bool> {
        self.repo_adapter()
            .restore_repo_file(relative_repo_path, destination_path)
            .await
    }
}

#[async_trait]
impl RepoSigningConfigLoader for RepoSigningDeps {
    async fn load_effective_daemon_config(&self) -> anyhow::Result<DaemonConfig> {
        load_effective_daemon_config_from_store(&self.base_config, &self.store).await
    }

    async fn sync_keyring_from_runtime_settings(
        &self,
        config: &mut DaemonConfig,
    ) -> anyhow::Result<()> {
        sync_keyring_from_runtime_settings(&self.store, config).await
    }
}

#[async_trait]
impl RepoSigningInspector for RepoSigningDeps {
    async fn inspect_repo_signing_status(
        &self,
        config: &DaemonConfig,
    ) -> anyhow::Result<RepoSigningStatusView> {
        self.repo_adapter()
            .inspect_repo_signing_status(config)
            .await
    }
}

#[async_trait]
impl RepoSigningProgressReader for RepoSigningDeps {
    async fn load_repo_signing_reconcile_progress(
        &self,
    ) -> Option<synforge_core::api::RepoSigningReconcileProgressView> {
        self.signing_progress.load().await
    }
}

#[async_trait]
impl RepoSigningSettingsWriter for RepoSigningDeps {
    async fn persist_repo_signing_settings(
        &self,
        update: RepoSigningSettingsUpdate,
    ) -> anyhow::Result<()> {
        let mut settings = BTreeMap::new();
        if let Some(enabled) = update.signing_enabled {
            settings.insert(
                "signing_enabled".to_string(),
                serde_json::Value::Bool(enabled),
            );
        }
        if let Some(key_id) = update.signing_key_id {
            settings.insert(
                "signing_key_id".to_string(),
                key_id
                    .map(serde_json::Value::String)
                    .unwrap_or(serde_json::Value::Null),
            );
        }
        if let Some(armored_private_key) = update.armored_private_key {
            settings.insert(
                RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED.to_string(),
                armored_private_key
                    .map(serde_json::Value::String)
                    .unwrap_or(serde_json::Value::Null),
            );
        }
        if settings.is_empty() {
            return Ok(());
        }
        PostgresRuntimeSettingsStore::new(self.store.clone())
            .upsert(settings)
            .await
    }
}

#[async_trait]
impl RepoSigningCommandRunner for RepoSigningDeps {
    async fn generate_new_managed_signing_key(
        &self,
        config: &DaemonConfig,
    ) -> anyhow::Result<RepoSigningKeyIdentity> {
        self.repo_adapter()
            .generate_new_managed_signing_key(config)
            .await
    }

    async fn import_private_key(
        &self,
        config: &DaemonConfig,
        armored_private_key: &str,
    ) -> anyhow::Result<RepoSigningKeyIdentity> {
        self.repo_adapter()
            .import_private_key(config, armored_private_key)
            .await
    }

    async fn remove_all_keys(&self, config: &DaemonConfig) -> anyhow::Result<()> {
        self.repo_adapter().remove_all_keys(config).await
    }

    async fn export_private_key(
        &self,
        config: &DaemonConfig,
        key_id: &str,
    ) -> anyhow::Result<String> {
        self.repo_adapter().export_private_key(config, key_id).await
    }

    async fn resolve_configured_signing_identity(
        &self,
        config: &DaemonConfig,
    ) -> anyhow::Result<RepoSigningKeyIdentity> {
        self.repo_adapter()
            .resolve_configured_signing_identity(config)
            .await
    }

    async fn export_public_key(
        &self,
        config: &DaemonConfig,
        key_id: &str,
    ) -> anyhow::Result<String> {
        self.repo_adapter().export_public_key(config, key_id).await
    }

    async fn reconcile_repo_metadata_signature(&self, config: &DaemonConfig) -> anyhow::Result<()> {
        self.repo_adapter()
            .reconcile_repo_metadata_signature(config)
            .await
    }

    async fn add_signature_to_artifact(
        &self,
        config: &DaemonConfig,
        key_id: &str,
        artifact_path: &Path,
    ) -> anyhow::Result<()> {
        self.repo_adapter()
            .add_signature_to_artifact(config, key_id, artifact_path)
            .await
    }

    async fn remove_signature_from_artifact(&self, artifact_path: &Path) -> anyhow::Result<()> {
        self.repo_adapter()
            .remove_signature_from_artifact(artifact_path)
            .await
    }

    async fn recompute_artifact_metadata(
        &self,
        artifact_path: &Path,
    ) -> anyhow::Result<(String, u64)> {
        self.repo_adapter()
            .recompute_artifact_metadata(artifact_path)
            .await
    }
}

#[async_trait]
impl RepoSigningProgressWriter for RepoSigningDeps {
    async fn save_repo_signing_reconcile_progress(
        &self,
        progress: synforge_core::api::RepoSigningReconcileProgressView,
    ) {
        self.signing_progress.save(progress).await;
    }
}

#[async_trait]
impl RepoArtifactCatalog for RepoSigningDeps {
    async fn count_all_published_repo_files(&self) -> anyhow::Result<u64> {
        self.repo_store().count_all_published_repo_files().await
    }

    async fn list_all_published_repo_files(
        &self,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        self.repo_store()
            .list_all_published_repo_files(limit, offset)
            .await
    }

    async fn update_build_artifact_metadata(
        &self,
        artifact_id: Uuid,
        sha256: String,
        size_bytes: u64,
    ) -> anyhow::Result<()> {
        self.repo_store()
            .update_build_artifact_metadata(artifact_id, sha256, size_bytes)
            .await
    }

    async fn upsert_artifact_signatures(
        &self,
        signatures: Vec<ArtifactSignature>,
    ) -> anyhow::Result<()> {
        self.repo_store()
            .upsert_artifact_signatures(signatures)
            .await
    }

    async fn sync_repo_tree(&self, repo_root: &Path) -> anyhow::Result<()> {
        self.repo_adapter().sync_repo_tree(repo_root).await
    }
}

impl SynforgeService {
    pub(super) fn repo_store(&self) -> PostgresRepoStore {
        PostgresRepoStore::new(self.store.clone())
    }

    pub(super) fn repo_signing_deps(&self) -> RepoSigningDeps {
        RepoSigningDeps {
            store: self.store.clone(),
            base_config: self.config.clone(),
            signing_progress: self.signing_reconcile_progress.clone(),
        }
    }
}
