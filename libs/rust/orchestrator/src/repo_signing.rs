use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::Context;
use sha2::{Digest, Sha256};
use synforge_core::{
    config::{DEFAULT_SIGNING_PUBLIC_KEY_NAME, DaemonConfig},
    model::{ArtifactKind, ArtifactSignature, ArtifactSigningStatus, WorkerBuildResult, now_utc},
    package::PackageDefinition,
};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedSigningKey {
    pub key_id: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSigningStatus {
    pub enabled: bool,
    pub configured_key_id: Option<String>,
    pub keyring_dir: PathBuf,
    pub key_present: bool,
    pub active_fingerprint: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct RepoSigningManager;

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

    pub async fn sign_worker_artifacts_for_publication(
        &self,
        config: &DaemonConfig,
        package: &PackageDefinition,
        worker_result: &mut WorkerBuildResult,
    ) -> Vec<ArtifactSignature> {
        let signed_at = now_utc();

        if !config.signing_enabled {
            return worker_result
                .artifacts
                .iter()
                .map(|artifact| ArtifactSignature {
                    artifact_id: artifact.id,
                    status: ArtifactSigningStatus::Skipped,
                    signed_at: None,
                    key_id: None,
                    fingerprint: None,
                    error_message: Some("repository signing disabled".to_string()),
                })
                .collect();
        }

        let Some(key_id) = config.signing_key_id.as_deref() else {
            return worker_result
                .artifacts
                .iter()
                .map(|artifact| {
                    signature_failed(artifact.id, "missing signing key id in configuration")
                })
                .collect();
        };

        let fingerprint = match self
            .first_secret_key_fingerprint(config, Some(key_id))
            .await
        {
            Ok(Some(fingerprint)) => fingerprint,
            Ok(None) => {
                return worker_result
                    .artifacts
                    .iter()
                    .map(|artifact| {
                        signature_failed(
                            artifact.id,
                            format!("configured signing key not found in keyring: {}", key_id),
                        )
                    })
                    .collect();
            }
            Err(error) => {
                return worker_result
                    .artifacts
                    .iter()
                    .map(|artifact| {
                        signature_failed(
                            artifact.id,
                            format!("failed to inspect signing keyring: {}", error),
                        )
                    })
                    .collect();
            }
        };

        let keyring_dir = match self.ensure_keyring_dir(config).await {
            Ok(dir) => dir,
            Err(error) => {
                return worker_result
                    .artifacts
                    .iter()
                    .map(|artifact| {
                        signature_failed(
                            artifact.id,
                            format!("failed to initialize signing keyring: {}", error),
                        )
                    })
                    .collect();
            }
        };

        let mut signatures = Vec::with_capacity(worker_result.artifacts.len());
        for artifact in &mut worker_result.artifacts {
            if !is_publish_signable_artifact(package, artifact.kind) {
                signatures.push(ArtifactSignature {
                    artifact_id: artifact.id,
                    status: ArtifactSigningStatus::Skipped,
                    signed_at: None,
                    key_id: Some(key_id.to_string()),
                    fingerprint: Some(fingerprint.clone()),
                    error_message: Some(
                        "artifact is not configured for publication/signing".to_string(),
                    ),
                });
                continue;
            }

            let artifact_path = config
                .runtime_paths()
                .job_artifacts_dir(worker_result.job_id)
                .join(artifact.storage_path());
            let sign_result = run_rpm_addsign(&keyring_dir, key_id, artifact_path.as_path()).await;
            match sign_result {
                Ok(()) => match recompute_artifact_metadata(artifact_path.as_path()).await {
                    Ok((sha256, size_bytes)) => {
                        artifact.sha256 = sha256;
                        artifact.size_bytes = size_bytes;
                        signatures.push(ArtifactSignature {
                            artifact_id: artifact.id,
                            status: ArtifactSigningStatus::Signed,
                            signed_at: Some(signed_at),
                            key_id: Some(key_id.to_string()),
                            fingerprint: Some(fingerprint.clone()),
                            error_message: None,
                        });
                    }
                    Err(error) => signatures.push(signature_failed(
                        artifact.id,
                        format!("signed artifact metadata refresh failed: {}", error),
                    )),
                },
                Err(error) => signatures.push(signature_failed(
                    artifact.id,
                    format!("rpm signing failed: {}", error),
                )),
            }
        }
        signatures
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

    pub async fn add_signature_to_artifact(
        &self,
        config: &DaemonConfig,
        key_id: &str,
        artifact_path: &Path,
    ) -> anyhow::Result<()> {
        let keyring_dir = self.ensure_keyring_dir(config).await?;
        run_rpm_addsign(&keyring_dir, key_id, artifact_path).await
    }

    pub async fn remove_signature_from_artifact(&self, artifact_path: &Path) -> anyhow::Result<()> {
        run_rpm_delsign(artifact_path).await
    }

    pub async fn recompute_artifact_metadata(&self, path: &Path) -> anyhow::Result<(String, u64)> {
        recompute_artifact_metadata(path).await
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

    pub async fn reconcile_repo_metadata_signature(
        &self,
        config: &DaemonConfig,
        repo_dir: &Path,
    ) -> anyhow::Result<()> {
        self.clear_repo_metadata_signature(repo_dir).await?;
        if config.signing_enabled {
            self.sync_repo_public_key(config, repo_dir).await
        } else {
            self.clear_repo_public_key(config, repo_dir).await
        }
    }

    fn keyring_dir(&self, config: &DaemonConfig) -> PathBuf {
        config.runtime_paths().signing_root().join("gnupg")
    }

    pub fn repo_public_key_path(&self, config: &DaemonConfig, repo_dir: &Path) -> PathBuf {
        let _ = config;
        repo_dir.join(DEFAULT_SIGNING_PUBLIC_KEY_NAME)
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

    async fn ensure_keyring_dir(&self, config: &DaemonConfig) -> anyhow::Result<PathBuf> {
        let keyring_dir = self.keyring_dir(config);
        tokio::fs::create_dir_all(&keyring_dir)
            .await
            .with_context(|| {
                format!("failed to create signing keyring {}", keyring_dir.display())
            })?;
        set_owner_only_permissions(&keyring_dir)?;
        Ok(keyring_dir)
    }

    async fn sync_repo_public_key(
        &self,
        config: &DaemonConfig,
        repo_dir: &Path,
    ) -> anyhow::Result<()> {
        let (key_id, _) = self.resolve_configured_signing_identity(config).await?;
        let public_key = self.export_public_key(config, key_id.as_str()).await?;
        let public_key_path = self.repo_public_key_path(config, repo_dir);
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

    async fn clear_repo_public_key(
        &self,
        config: &DaemonConfig,
        repo_dir: &Path,
    ) -> anyhow::Result<()> {
        let public_key_path = self.repo_public_key_path(config, repo_dir);
        match tokio::fs::remove_file(&public_key_path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    async fn first_secret_key_fingerprint(
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
}

fn signature_failed(artifact_id: uuid::Uuid, message: impl Into<String>) -> ArtifactSignature {
    ArtifactSignature {
        artifact_id,
        status: ArtifactSigningStatus::Failed,
        signed_at: None,
        key_id: None,
        fingerprint: None,
        error_message: Some(message.into()),
    }
}

fn is_publish_signable_artifact(package: &PackageDefinition, kind: ArtifactKind) -> bool {
    match kind {
        ArtifactKind::Rpm => true,
        ArtifactKind::Srpm => package.publish_srpm,
        ArtifactKind::Debuginfo | ArtifactKind::Debugsource => package.publish_debuginfo,
        ArtifactKind::Log | ArtifactKind::Other => false,
    }
}

fn parse_first_fingerprint(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let mut fields = line.split(':');
        if fields.next() != Some("fpr") {
            return None;
        }
        let fingerprint = fields.nth(8)?;
        if fingerprint.is_empty() {
            None
        } else {
            Some(fingerprint.to_string())
        }
    })
}

async fn run_gpg(keyring_dir: &Path, args: &[&str]) -> anyhow::Result<()> {
    let output = Command::new("gpg")
        .arg("--homedir")
        .arg(keyring_dir)
        .args(args)
        .output()
        .await
        .context("failed to execute gpg")?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "gpg command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

async fn run_gpg_with_input(keyring_dir: &Path, args: &[&str], input: &[u8]) -> anyhow::Result<()> {
    let mut child = Command::new("gpg")
        .arg("--homedir")
        .arg(keyring_dir)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn gpg")?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input)
            .await
            .context("failed to write gpg stdin")?;
    }
    let output = child
        .wait_with_output()
        .await
        .context("failed to wait for gpg process")?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "gpg command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

async fn run_gpg_capture_stdout(keyring_dir: &Path, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("gpg")
        .arg("--homedir")
        .arg(keyring_dir)
        .args(args)
        .output()
        .await
        .context("failed to execute gpg")?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "gpg command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout).context("gpg output was not valid utf-8")
}

async fn run_rpm_addsign(
    keyring_dir: &Path,
    key_id: &str,
    artifact_path: &Path,
) -> anyhow::Result<()> {
    let define_gpg_name = format!("_gpg_name {}", key_id);
    let define_gpg_path = format!("_gpg_path {}", keyring_dir.display());
    let output = match run_rpm_resign_command(
        "rpmsign",
        define_gpg_name.as_str(),
        define_gpg_path.as_str(),
        artifact_path,
    )
    .await
    {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => run_rpm_resign_command(
            "rpm",
            define_gpg_name.as_str(),
            define_gpg_path.as_str(),
            artifact_path,
        )
        .await
        .context("failed to execute rpmsign/rpm --resign")?,
        Err(error) => return Err(error).context("failed to execute rpmsign --resign"),
    };

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "rpm signing failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

async fn run_rpm_resign_command(
    binary: &str,
    define_gpg_name: &str,
    define_gpg_path: &str,
    artifact_path: &Path,
) -> std::io::Result<std::process::Output> {
    Command::new(binary)
        .arg("--define")
        .arg(define_gpg_name)
        .arg("--define")
        .arg(define_gpg_path)
        .arg("--define")
        .arg("_signature gpg")
        .arg("--define")
        .arg("__gpg /usr/bin/gpg")
        .arg("--resign")
        .arg(artifact_path)
        .output()
        .await
}

async fn run_rpm_delsign(artifact_path: &Path) -> anyhow::Result<()> {
    let output = Command::new("rpm")
        .arg("--delsign")
        .arg(artifact_path)
        .output()
        .await
        .context("failed to execute rpm --delsign")?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "rpm --delsign failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

async fn recompute_artifact_metadata(path: &Path) -> anyhow::Result<(String, u64)> {
    let bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("failed to read artifact {}", path.display()))?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    Ok((sha256, bytes.len() as u64))
}

#[cfg(unix)]
fn set_owner_only_permissions(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .with_context(|| format!("failed to set permissions on {}", path.display()))
}

#[cfg(not(unix))]
fn set_owner_only_permissions(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_first_fingerprint;

    #[test]
    fn extracts_first_fingerprint_from_colon_output() {
        let output = "sec:-:255:22:ABCDEF1234:1710000000:0::::::scESC:::+:::23::0:\n\
fpr:::::::::0123456789ABCDEF0123456789ABCDEF01234567:\n\
uid:-::::1710000000::ABCDEF1234567890::Example <example@example.com>::::::::::0:\n";
        assert_eq!(
            parse_first_fingerprint(output),
            Some("0123456789ABCDEF0123456789ABCDEF01234567".to_string())
        );
    }

    #[test]
    fn returns_none_when_no_fingerprint_exists() {
        let output = "sec:-:255:22:ABCDEF1234:1710000000:0::::::scESC:::+:::23::0:\n";
        assert_eq!(parse_first_fingerprint(output), None);
    }
}
