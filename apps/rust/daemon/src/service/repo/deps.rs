use std::{collections::BTreeMap, path::Path};

use async_trait::async_trait;
use delegate::delegate;
use synforge_core::{
    api::RepoSigningStatusView,
    config::{DaemonConfig, RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED},
    model::{ArtifactSignature, PublishedRepoFile},
};
use synforge_database::{
    DieselStore, repo::PostgresRepoStore, runtime_settings::PostgresRuntimeSettingsStore,
};
use synforge_publish::{
    RepoArtifactCatalog, RepoSigningCommandRunner, RepoSigningConfigLoader, RepoSigningInspector,
    RepoSigningKeyIdentity, RepoSigningProgressReader, RepoSigningProgressWriter,
    RepoSigningSettingsUpdate, RepoSigningSettingsWriter,
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
    fn repo_store(&self) -> PostgresRepoStore {
        PostgresRepoStore::new(self.store.clone())
    }

    fn signing_manager(&self) -> synforge_publish::RepoSigningManager {
        synforge_publish::RepoSigningManager
    }

    delegate! {
        to self.repo_store() {
            #[call(count_all_published_repo_files)]
            async fn load_published_repo_file_count(&self) -> anyhow::Result<u64>;

            #[call(list_all_published_repo_files)]
            async fn load_published_repo_files(
                &self,
                limit: usize,
                offset: usize,
            ) -> anyhow::Result<Vec<PublishedRepoFile>>;

            #[call(update_build_artifact_metadata)]
            async fn persist_build_artifact_metadata(
                &self,
                artifact_id: Uuid,
                sha256: String,
                size_bytes: u64,
            ) -> anyhow::Result<()>;

            #[call(upsert_artifact_signatures)]
            async fn persist_artifact_signatures(
                &self,
                signatures: Vec<ArtifactSignature>,
            ) -> anyhow::Result<()>;
        }
    }

    delegate! {
        to self.signing_manager() {
            #[call(remove_all_keys)]
            async fn clear_signing_keys(&self, config: &DaemonConfig) -> anyhow::Result<()>;

            #[call(export_private_key)]
            async fn load_private_key(&self, config: &DaemonConfig, key_id: &str) -> anyhow::Result<String>;

            #[call(export_public_key)]
            async fn load_public_key(&self, config: &DaemonConfig, key_id: &str) -> anyhow::Result<String>;

            #[call(add_signature_to_artifact)]
            async fn sign_artifact(
                &self,
                config: &DaemonConfig,
                key_id: &str,
                artifact_path: &Path,
            ) -> anyhow::Result<()>;

            #[call(remove_signature_from_artifact)]
            async fn unsign_artifact(&self, artifact_path: &Path) -> anyhow::Result<()>;

            #[call(recompute_artifact_metadata)]
            async fn load_artifact_metadata(&self, artifact_path: &Path) -> anyhow::Result<(String, u64)>;
        }
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
        let status = self.signing_manager().status(config).await?;
        Ok(RepoSigningStatusView {
            enabled: status.enabled,
            configured_key_id: status.configured_key_id,
            keyring_dir: status.keyring_dir,
            repo_public_key_path: self
                .signing_manager()
                .repo_public_key_path(config.runtime_paths().repo_dir()),
            key_present: status.key_present,
            active_fingerprint: status.active_fingerprint,
            can_export_private_key: false,
        })
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
        let key = self.signing_manager().generate_new_managed_signing_key(config).await?;
        Ok(RepoSigningKeyIdentity {
            key_id: key.key_id,
            fingerprint: key.fingerprint,
        })
    }

    async fn import_private_key(
        &self,
        config: &DaemonConfig,
        armored_private_key: &str,
    ) -> anyhow::Result<RepoSigningKeyIdentity> {
        let key = self
            .signing_manager()
            .import_private_key(config, armored_private_key)
            .await?;
        Ok(RepoSigningKeyIdentity {
            key_id: key.key_id,
            fingerprint: key.fingerprint,
        })
    }

    async fn remove_all_keys(&self, config: &DaemonConfig) -> anyhow::Result<()> {
        self.clear_signing_keys(config).await
    }

    async fn export_private_key(
        &self,
        config: &DaemonConfig,
        key_id: &str,
    ) -> anyhow::Result<String> {
        self.load_private_key(config, key_id).await
    }

    async fn resolve_configured_signing_identity(
        &self,
        config: &DaemonConfig,
    ) -> anyhow::Result<RepoSigningKeyIdentity> {
        let (key_id, fingerprint) = self
            .signing_manager()
            .resolve_configured_signing_identity(config)
            .await?;
        Ok(RepoSigningKeyIdentity {
            key_id,
            fingerprint,
        })
    }

    async fn export_public_key(
        &self,
        config: &DaemonConfig,
        key_id: &str,
    ) -> anyhow::Result<String> {
        self.load_public_key(config, key_id).await
    }

    async fn reconcile_repo_metadata_signature(&self, config: &DaemonConfig) -> anyhow::Result<()> {
        self.signing_manager()
            .reconcile_repo_metadata_signature(config, config.runtime_paths().repo_dir())
            .await
    }

    async fn add_signature_to_artifact(
        &self,
        config: &DaemonConfig,
        key_id: &str,
        artifact_path: &Path,
    ) -> anyhow::Result<()> {
        self.sign_artifact(config, key_id, artifact_path).await
    }

    async fn remove_signature_from_artifact(&self, artifact_path: &Path) -> anyhow::Result<()> {
        self.unsign_artifact(artifact_path).await
    }

    async fn recompute_artifact_metadata(
        &self,
        artifact_path: &Path,
    ) -> anyhow::Result<(String, u64)> {
        self.load_artifact_metadata(artifact_path).await
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
        self.load_published_repo_file_count().await
    }

    async fn list_all_published_repo_files(
        &self,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        self.load_published_repo_files(limit, offset).await
    }

    async fn update_build_artifact_metadata(
        &self,
        artifact_id: Uuid,
        sha256: String,
        size_bytes: u64,
    ) -> anyhow::Result<()> {
        self.persist_build_artifact_metadata(artifact_id, sha256, size_bytes)
            .await
    }

    async fn upsert_artifact_signatures(
        &self,
        signatures: Vec<ArtifactSignature>,
    ) -> anyhow::Result<()> {
        self.persist_artifact_signatures(signatures).await
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
