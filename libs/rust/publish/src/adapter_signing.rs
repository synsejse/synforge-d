use std::path::Path;

use async_trait::async_trait;
use synforge_core::{api::RepoSigningStatusView, config::DaemonConfig};

use crate::{
    RepoSigningCommandRunner, RepoSigningInspector, RepoSigningKeyIdentity, RepoSigningManager,
    RuntimeRepoAdapter,
};

#[async_trait]
impl RepoSigningInspector for RuntimeRepoAdapter {
    async fn inspect_repo_signing_status(
        &self,
        config: &DaemonConfig,
    ) -> anyhow::Result<RepoSigningStatusView> {
        let manager = RepoSigningManager;
        let status = manager.status(config).await?;
        Ok(RepoSigningStatusView {
            enabled: status.enabled,
            configured_key_id: status.configured_key_id,
            keyring_dir: status.keyring_dir,
            repo_public_key_path: manager
                .repo_public_key_path(config, config.runtime_paths().repo_dir()),
            key_present: status.key_present,
            active_fingerprint: status.active_fingerprint,
            can_export_private_key: false,
        })
    }
}

#[async_trait]
impl RepoSigningCommandRunner for RuntimeRepoAdapter {
    async fn generate_new_managed_signing_key(
        &self,
        config: &DaemonConfig,
    ) -> anyhow::Result<RepoSigningKeyIdentity> {
        let manager = RepoSigningManager;
        let key = manager.generate_new_managed_signing_key(config).await?;
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
        let manager = RepoSigningManager;
        let key = manager
            .import_private_key(config, armored_private_key)
            .await?;
        Ok(RepoSigningKeyIdentity {
            key_id: key.key_id,
            fingerprint: key.fingerprint,
        })
    }

    async fn remove_all_keys(&self, config: &DaemonConfig) -> anyhow::Result<()> {
        let manager = RepoSigningManager;
        manager.remove_all_keys(config).await
    }

    async fn export_private_key(
        &self,
        config: &DaemonConfig,
        key_id: &str,
    ) -> anyhow::Result<String> {
        let manager = RepoSigningManager;
        manager.export_private_key(config, key_id).await
    }

    async fn resolve_configured_signing_identity(
        &self,
        config: &DaemonConfig,
    ) -> anyhow::Result<RepoSigningKeyIdentity> {
        let manager = RepoSigningManager;
        let (key_id, fingerprint) = manager.resolve_configured_signing_identity(config).await?;
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
        let manager = RepoSigningManager;
        manager.export_public_key(config, key_id).await
    }

    async fn reconcile_repo_metadata_signature(&self, config: &DaemonConfig) -> anyhow::Result<()> {
        let manager = RepoSigningManager;
        manager
            .reconcile_repo_metadata_signature(config, config.runtime_paths().repo_dir())
            .await
    }

    async fn add_signature_to_artifact(
        &self,
        config: &DaemonConfig,
        key_id: &str,
        artifact_path: &Path,
    ) -> anyhow::Result<()> {
        let manager = RepoSigningManager;
        manager
            .add_signature_to_artifact(config, key_id, artifact_path)
            .await
    }

    async fn remove_signature_from_artifact(&self, artifact_path: &Path) -> anyhow::Result<()> {
        let manager = RepoSigningManager;
        manager.remove_signature_from_artifact(artifact_path).await
    }

    async fn recompute_artifact_metadata(
        &self,
        artifact_path: &Path,
    ) -> anyhow::Result<(String, u64)> {
        let manager = RepoSigningManager;
        manager.recompute_artifact_metadata(artifact_path).await
    }
}
