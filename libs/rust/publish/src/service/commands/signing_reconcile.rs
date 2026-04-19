use synforge_core::{
    api::{RepoSigningReconcileMode, RepoSigningReconcileProgressView, RepoSigningReconcileState},
    config::DaemonConfig,
    model::{ArtifactKind, ArtifactSignature, ArtifactSigningStatus, now_utc},
};
use uuid::Uuid;

use super::{RepoArtifactCatalog, RepoSigningCommandRunner, RepoSigningProgressWriter};
use crate::service::RepoSigningConfigLoader;

pub async fn reconcile_existing_artifacts<D>(
    deps: &D,
    config: &DaemonConfig,
    mode: RepoSigningReconcileMode,
) -> anyhow::Result<()>
where
    D: RepoSigningConfigLoader
        + RepoSigningCommandRunner
        + RepoArtifactCatalog
        + RepoSigningProgressWriter
        + Send
        + Sync,
{
    let mut config = config.clone();
    deps.sync_keyring_from_runtime_settings(&mut config).await?;
    let operation_id = Uuid::now_v7();
    let total_artifacts = deps.count_all_published_repo_files().await?;
    deps.save_repo_signing_reconcile_progress(RepoSigningReconcileProgressView {
        operation_id,
        mode: mode.clone(),
        state: RepoSigningReconcileState::Running,
        total_artifacts,
        processed_artifacts: 0,
        failed_artifacts: 0,
        message: None,
    })
    .await;

    let (key_id, fingerprint) = if mode == RepoSigningReconcileMode::Sign {
        let identity = deps.resolve_configured_signing_identity(&config).await?;
        (Some(identity.key_id), Some(identity.fingerprint))
    } else {
        (None, None)
    };

    let mut processed_artifacts = 0_u64;
    let mut failed_artifacts = 0_u64;
    let mut offset = 0_usize;
    let mut signatures = Vec::new();
    const PAGE_SIZE: usize = 200;

    let result: anyhow::Result<()> = async {
        loop {
            let files = deps
                .list_all_published_repo_files(PAGE_SIZE, offset)
                .await?;
            if files.is_empty() {
                break;
            }
            offset += files.len();
            for file in files {
                processed_artifacts += 1;
                if !is_signable_existing_artifact(file.kind) {
                    deps.save_repo_signing_reconcile_progress(RepoSigningReconcileProgressView {
                        operation_id,
                        mode: mode.clone(),
                        state: RepoSigningReconcileState::Running,
                        total_artifacts,
                        processed_artifacts,
                        failed_artifacts,
                        message: None,
                    })
                    .await;
                    continue;
                }

                let artifact_path = config.runtime_paths().repo_dir().join(&file.path);
                if !tokio::fs::try_exists(&artifact_path).await? {
                    failed_artifacts += 1;
                    signatures.push(signature_failed(
                        file.artifact_id,
                        format!(
                            "published repository artifact is missing: {}",
                            artifact_path.display()
                        ),
                    ));
                    deps.save_repo_signing_reconcile_progress(RepoSigningReconcileProgressView {
                        operation_id,
                        mode: mode.clone(),
                        state: RepoSigningReconcileState::Running,
                        total_artifacts,
                        processed_artifacts,
                        failed_artifacts,
                        message: None,
                    })
                    .await;
                    continue;
                }

                let signature_result = match mode {
                    RepoSigningReconcileMode::Sign => {
                        let Some(key_id) = key_id.as_deref() else {
                            return Err(anyhow::anyhow!("missing signing key id"));
                        };
                        deps.add_signature_to_artifact(&config, key_id, artifact_path.as_path())
                            .await
                    }
                    RepoSigningReconcileMode::Unsign => {
                        deps.remove_signature_from_artifact(artifact_path.as_path())
                            .await
                    }
                };

                let signature = match signature_result {
                    Ok(()) => match deps
                        .recompute_artifact_metadata(artifact_path.as_path())
                        .await
                    {
                        Ok((sha256, size_bytes)) => {
                            if let Err(error) = deps
                                .update_build_artifact_metadata(
                                    file.artifact_id,
                                    sha256,
                                    size_bytes,
                                )
                                .await
                            {
                                failed_artifacts += 1;
                                signature_failed(
                                    file.artifact_id,
                                    format!("failed to persist artifact metadata: {}", error),
                                )
                            } else if mode == RepoSigningReconcileMode::Sign {
                                ArtifactSignature {
                                    artifact_id: file.artifact_id,
                                    status: ArtifactSigningStatus::Signed,
                                    signed_at: Some(now_utc()),
                                    key_id: key_id.clone(),
                                    fingerprint: fingerprint.clone(),
                                    error_message: None,
                                }
                            } else {
                                ArtifactSignature {
                                    artifact_id: file.artifact_id,
                                    status: ArtifactSigningStatus::Skipped,
                                    signed_at: None,
                                    key_id: None,
                                    fingerprint: None,
                                    error_message: Some("artifact signature removed".to_string()),
                                }
                            }
                        }
                        Err(error) => {
                            failed_artifacts += 1;
                            signature_failed(
                                file.artifact_id,
                                format!("signed artifact metadata refresh failed: {}", error),
                            )
                        }
                    },
                    Err(error) => {
                        failed_artifacts += 1;
                        let action = if mode == RepoSigningReconcileMode::Sign {
                            "artifact signing failed"
                        } else {
                            "artifact unsigning failed"
                        };
                        signature_failed(file.artifact_id, format!("{}: {}", action, error))
                    }
                };

                signatures.push(signature);
                if signatures.len() >= 200 {
                    deps.upsert_artifact_signatures(std::mem::take(&mut signatures))
                        .await?;
                }
                deps.save_repo_signing_reconcile_progress(RepoSigningReconcileProgressView {
                    operation_id,
                    mode: mode.clone(),
                    state: RepoSigningReconcileState::Running,
                    total_artifacts,
                    processed_artifacts,
                    failed_artifacts,
                    message: None,
                })
                .await;
            }
        }

        if !signatures.is_empty() {
            deps.upsert_artifact_signatures(signatures).await?;
        }
        deps.reconcile_repo_metadata_signature(&config).await?;
        Ok(())
    }
    .await;

    match result {
        Ok(()) => {
            deps.save_repo_signing_reconcile_progress(RepoSigningReconcileProgressView {
                operation_id,
                mode,
                state: RepoSigningReconcileState::Completed,
                total_artifacts,
                processed_artifacts,
                failed_artifacts,
                message: None,
            })
            .await;
            Ok(())
        }
        Err(error) => {
            deps.save_repo_signing_reconcile_progress(RepoSigningReconcileProgressView {
                operation_id,
                mode,
                state: RepoSigningReconcileState::Failed,
                total_artifacts,
                processed_artifacts,
                failed_artifacts,
                message: Some(error.to_string()),
            })
            .await;
            Err(error)
        }
    }
}

fn signature_failed(artifact_id: Uuid, message: impl Into<String>) -> ArtifactSignature {
    ArtifactSignature {
        artifact_id,
        status: ArtifactSigningStatus::Failed,
        signed_at: None,
        key_id: None,
        fingerprint: None,
        error_message: Some(message.into()),
    }
}

fn is_signable_existing_artifact(kind: ArtifactKind) -> bool {
    matches!(
        kind,
        ArtifactKind::Rpm
            | ArtifactKind::Srpm
            | ArtifactKind::Debuginfo
            | ArtifactKind::Debugsource
    )
}
