use std::path::{Path, PathBuf};

use anyhow::Context;
use synforge_core::{config::DaemonConfig, constants::DEFAULT_SIGNING_PUBLIC_KEY_NAME};

use super::RepoSigningManager;
use crate::repo_manager::discover_repo_targets;

impl RepoSigningManager {
    pub async fn reconcile_repo_metadata_signature(
        &self,
        config: &DaemonConfig,
        repo_dir: &Path,
    ) -> anyhow::Result<()> {
        self.clear_repo_metadata_signature(repo_dir).await?;
        for target in discover_repo_targets(repo_dir).await? {
            let target_repo_dir = repo_dir.join(target.repo_subdir());
            self.clear_repo_metadata_signature(&target_repo_dir).await?;
        }
        if config.signing_enabled {
            self.sync_repo_public_key(config, repo_dir).await
        } else {
            self.clear_repo_public_key(repo_dir).await
        }
    }

    pub fn repo_public_key_path(&self, repo_dir: &Path) -> PathBuf {
        repo_dir.join(DEFAULT_SIGNING_PUBLIC_KEY_NAME)
    }

    async fn sync_repo_public_key(
        &self,
        config: &DaemonConfig,
        repo_dir: &Path,
    ) -> anyhow::Result<()> {
        let (key_id, _) = self.resolve_configured_signing_identity(config).await?;
        let public_key = self.export_public_key(config, key_id.as_str()).await?;
        let public_key_path = self.repo_public_key_path(repo_dir);
        if let Some(parent) = public_key_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&public_key_path, public_key)
            .await
            .with_context(|| {
                format!(
                    "failed to write exported repository signing key {}",
                    public_key_path.display()
                )
            })?;
        Ok(())
    }

    async fn clear_repo_metadata_signature(&self, repo_dir: &Path) -> anyhow::Result<()> {
        let signature_path = repo_dir.join("repodata/repomd.xml.asc");
        match tokio::fs::remove_file(&signature_path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    async fn clear_repo_public_key(&self, repo_dir: &Path) -> anyhow::Result<()> {
        let public_key_path = self.repo_public_key_path(repo_dir);
        match tokio::fs::remove_file(&public_key_path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}
