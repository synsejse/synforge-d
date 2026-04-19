use std::path::PathBuf;

use anyhow::Context;
use synforge_core::config::DaemonConfig;
use uuid::Uuid;

use super::commands::{
    parse_first_fingerprint, run_gpg, run_gpg_capture_stdout, run_gpg_with_input,
    set_owner_only_permissions,
};
use super::{ImportedSigningKey, RepoSigningManager, RepoSigningStatus};

impl RepoSigningManager {
    pub async fn generate_new_managed_signing_key(
        &self,
        config: &DaemonConfig,
    ) -> anyhow::Result<ImportedSigningKey> {
        self.remove_all_keys(config).await?;
        self.create_managed_signing_key(config).await
    }

    pub async fn ensure_managed_signing_key(
        &self,
        config: &DaemonConfig,
    ) -> anyhow::Result<ImportedSigningKey> {
        if let Some(configured_key_id) = config.signing_key_id.as_deref()
            && let Some(fingerprint) = self
                .first_secret_key_fingerprint(config, Some(configured_key_id))
                .await?
        {
            return Ok(ImportedSigningKey {
                key_id: configured_key_id.to_string(),
                fingerprint,
            });
        }

        if let Some(fingerprint) = self.first_secret_key_fingerprint(config, None).await? {
            return Ok(ImportedSigningKey {
                key_id: fingerprint.clone(),
                fingerprint,
            });
        }

        self.create_managed_signing_key(config).await
    }

    pub async fn import_private_key(
        &self,
        config: &DaemonConfig,
        armored_private_key: &str,
    ) -> anyhow::Result<ImportedSigningKey> {
        let keyring_dir = self.ensure_keyring_dir(config).await?;
        run_gpg_with_input(
            &keyring_dir,
            &["--batch", "--yes", "--import"],
            armored_private_key.as_bytes(),
        )
        .await?;

        let fingerprint = self
            .first_secret_key_fingerprint(config, None)
            .await?
            .ok_or_else(|| anyhow::anyhow!("no secret key found after import"))?;

        Ok(ImportedSigningKey {
            key_id: fingerprint.clone(),
            fingerprint,
        })
    }

    pub async fn remove_all_keys(&self, config: &DaemonConfig) -> anyhow::Result<()> {
        let keyring_dir = self.keyring_dir(config);
        if tokio::fs::try_exists(&keyring_dir).await? {
            tokio::fs::remove_dir_all(&keyring_dir)
                .await
                .with_context(|| {
                    format!("failed to remove signing keyring {}", keyring_dir.display())
                })?;
        }
        Ok(())
    }

    pub async fn status(&self, config: &DaemonConfig) -> anyhow::Result<RepoSigningStatus> {
        let keyring_dir = self.keyring_dir(config);
        let active_fingerprint = if let Some(configured_key_id) = config.signing_key_id.as_deref() {
            self.first_secret_key_fingerprint(config, Some(configured_key_id))
                .await?
        } else {
            self.first_secret_key_fingerprint(config, None).await?
        };

        Ok(RepoSigningStatus {
            enabled: config.signing_enabled,
            configured_key_id: config.signing_key_id.clone(),
            keyring_dir,
            key_present: active_fingerprint.is_some(),
            active_fingerprint,
        })
    }

    pub async fn export_private_key(
        &self,
        config: &DaemonConfig,
        key_id: &str,
    ) -> anyhow::Result<String> {
        let keyring_dir = self.ensure_keyring_dir(config).await?;
        run_gpg_capture_stdout(
            &keyring_dir,
            &[
                "--batch",
                "--yes",
                "--armor",
                "--export-secret-keys",
                key_id,
            ],
        )
        .await
    }

    pub async fn export_public_key(
        &self,
        config: &DaemonConfig,
        key_id: &str,
    ) -> anyhow::Result<String> {
        let keyring_dir = self.ensure_keyring_dir(config).await?;
        run_gpg_capture_stdout(
            &keyring_dir,
            &["--batch", "--yes", "--armor", "--export", key_id],
        )
        .await
    }

    pub(super) fn keyring_dir(&self, config: &DaemonConfig) -> PathBuf {
        config.runtime_paths().signing_root().join("gnupg")
    }

    pub(super) async fn ensure_keyring_dir(
        &self,
        config: &DaemonConfig,
    ) -> anyhow::Result<PathBuf> {
        let keyring_dir = self.keyring_dir(config);
        tokio::fs::create_dir_all(&keyring_dir)
            .await
            .with_context(|| {
                format!("failed to create signing keyring {}", keyring_dir.display())
            })?;
        set_owner_only_permissions(&keyring_dir).await?;
        Ok(keyring_dir)
    }

    pub(super) async fn first_secret_key_fingerprint(
        &self,
        config: &DaemonConfig,
        key_id: Option<&str>,
    ) -> anyhow::Result<Option<String>> {
        let keyring_dir = self.keyring_dir(config);
        if !tokio::fs::try_exists(&keyring_dir).await? {
            return Ok(None);
        }
        let mut args = vec![
            "--batch",
            "--with-colons",
            "--fingerprint",
            "--list-secret-keys",
        ];
        if let Some(key_id) = key_id {
            args.push(key_id);
        }
        let output = run_gpg_capture_stdout(&keyring_dir, &args).await?;
        Ok(parse_first_fingerprint(&output))
    }

    async fn create_managed_signing_key(
        &self,
        config: &DaemonConfig,
    ) -> anyhow::Result<ImportedSigningKey> {
        let keyring_dir = self.ensure_keyring_dir(config).await?;
        let key_nonce = Uuid::now_v7().simple().to_string();
        let key_identity = format!(
            "Synforge Repository Signing {} <synforge+{}@localhost>",
            key_nonce, key_nonce
        );
        run_gpg(
            &keyring_dir,
            &[
                "--batch",
                "--yes",
                "--pinentry-mode",
                "loopback",
                "--passphrase",
                "",
                "--quick-gen-key",
                key_identity.as_str(),
                "rsa3072",
                "sign",
                "0",
            ],
        )
        .await?;

        let fingerprint = self
            .first_secret_key_fingerprint(config, None)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("managed signing key generation completed without a usable key")
            })?;

        Ok(ImportedSigningKey {
            key_id: fingerprint.clone(),
            fingerprint,
        })
    }

    pub async fn resolve_configured_signing_identity(
        &self,
        config: &DaemonConfig,
    ) -> anyhow::Result<(String, String)> {
        let key_id = config.signing_key_id.as_deref().ok_or_else(|| {
            anyhow::anyhow!("signing_key_id must be configured when signing is enabled")
        })?;
        let fingerprint = self
            .first_secret_key_fingerprint(config, Some(key_id))
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "configured signing key was not found in keyring: {}",
                    key_id
                )
            })?;
        Ok((key_id.to_string(), fingerprint))
    }
}
