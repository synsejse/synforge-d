use std::path::Path;

use synforge_core::{
    config::DaemonConfig,
    model::{ArtifactKind, ArtifactSignature, ArtifactSigningStatus, WorkerBuildResult, now_utc},
    package::PackageDefinition,
};

use super::RepoSigningManager;
use super::commands::{recompute_artifact_metadata, run_rpm_addsign, run_rpm_delsign};

impl RepoSigningManager {
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
