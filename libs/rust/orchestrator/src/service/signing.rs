use super::SynforgeService;
use crate::RepoSigningManager;
use crate::db::RepoStore;
use std::collections::BTreeMap;
use synforge_core::{
    api::{
        ExportRepoSigningKeyResponse, ExportRepoSigningPublicKeyResponse,
        GenerateRepoSigningKeyResponse, ImportRepoSigningKeyRequest, ImportRepoSigningKeyResponse,
        RepoSigningReconcileMode, RepoSigningReconcileProgressResponse,
        RepoSigningReconcileProgressView, RepoSigningReconcileState, RepoSigningStatusResponse,
        RepoSigningStatusView, TestRepoSigningResponse, UpdateRepoSigningConfigRequest,
    },
    config::DaemonConfig,
    error::SynforgeError,
    model::{ArtifactKind, ArtifactSignature, ArtifactSigningStatus, UserAccount, now_utc},
};
use uuid::Uuid;

pub(crate) const RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED: &str = "signing_private_key_armored";

impl SynforgeService {
    pub async fn get_repo_signing_status(
        &self,
        current_user: &UserAccount,
    ) -> anyhow::Result<RepoSigningStatusResponse> {
        let mut config = self.load_effective_daemon_config().await?;
        self.sync_keyring_from_runtime_settings(&mut config).await?;
        let manager = RepoSigningManager;
        let status = manager.status(&config).await?;
        let can_export_private_key = self.is_bootstrap_admin_user(current_user.id).await?;
        Ok(RepoSigningStatusResponse {
            status: RepoSigningStatusView {
                enabled: status.enabled,
                configured_key_id: status.configured_key_id,
                keyring_dir: status.keyring_dir,
                repo_public_key_path: manager
                    .repo_public_key_path(&config, config.runtime_paths().repo_dir()),
                key_present: status.key_present,
                active_fingerprint: status.active_fingerprint,
                can_export_private_key,
            },
        })
    }

    pub async fn get_repo_signing_reconcile_progress(
        &self,
    ) -> anyhow::Result<RepoSigningReconcileProgressResponse> {
        let progress = self.signing_reconcile_progress.lock().await.clone();
        Ok(RepoSigningReconcileProgressResponse {
            operation: progress,
        })
    }

    pub async fn update_repo_signing_config(
        &self,
        request: UpdateRepoSigningConfigRequest,
        current_user_id: Uuid,
    ) -> anyhow::Result<RepoSigningStatusResponse> {
        let mut config = self.load_effective_daemon_config().await?;
        self.sync_keyring_from_runtime_settings(&mut config).await?;
        config.signing_enabled = request.enabled;

        if config.signing_enabled && config.signing_key_id.is_none() {
            return Err(anyhow::anyhow!(SynforgeError::BadRequest(
                "generate or import a private key before enabling repository signing".to_string(),
            )));
        }
        if config.signing_enabled {
            let manager = RepoSigningManager;
            let status = manager.status(&config).await?;
            if !status.key_present {
                return Err(anyhow::anyhow!(SynforgeError::BadRequest(
                    "no private key material is available; generate or import a key first"
                        .to_string(),
                )));
            }
        }

        let mut dynamic_settings = BTreeMap::new();
        dynamic_settings.insert(
            "signing_enabled".to_string(),
            serde_json::Value::Bool(config.signing_enabled),
        );
        self.store.upsert_runtime_settings(dynamic_settings).await?;

        let mode = if config.signing_enabled {
            RepoSigningReconcileMode::Sign
        } else {
            RepoSigningReconcileMode::Unsign
        };
        self.reconcile_existing_artifacts(&config, mode).await?;
        self.get_repo_signing_status_for_user_id(Some(current_user_id))
            .await
    }

    pub async fn generate_repo_signing_key(
        &self,
        current_user_id: Uuid,
    ) -> anyhow::Result<GenerateRepoSigningKeyResponse> {
        let mut config = self.load_effective_daemon_config().await?;
        self.sync_keyring_from_runtime_settings(&mut config).await?;
        if config.signing_enabled {
            return Err(anyhow::anyhow!(SynforgeError::BadRequest(
                "disable repository signing before generating a new key".to_string(),
            )));
        }
        let manager = RepoSigningManager;
        let managed = manager.generate_new_managed_signing_key(&config).await?;
        config.signing_key_id = Some(managed.key_id.clone());

        let mut dynamic_settings = BTreeMap::new();
        dynamic_settings.insert(
            "signing_key_id".to_string(),
            serde_json::Value::String(managed.key_id.clone()),
        );
        let armored_private_key = manager
            .export_private_key(&config, managed.key_id.as_str())
            .await?;
        dynamic_settings.insert(
            RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED.to_string(),
            serde_json::Value::String(armored_private_key),
        );
        self.store.upsert_runtime_settings(dynamic_settings).await?;

        if config.signing_enabled {
            manager
                .reconcile_repo_metadata_signature(&config, config.runtime_paths().repo_dir())
                .await?;
        }

        let status = self
            .get_repo_signing_status_for_user_id(Some(current_user_id))
            .await?;
        Ok(GenerateRepoSigningKeyResponse {
            key_id: managed.key_id,
            fingerprint: managed.fingerprint,
            status: status.status,
        })
    }

    pub async fn import_repo_signing_key(
        &self,
        request: ImportRepoSigningKeyRequest,
        current_user_id: Uuid,
    ) -> anyhow::Result<ImportRepoSigningKeyResponse> {
        let mut config = self.load_effective_daemon_config().await?;
        self.sync_keyring_from_runtime_settings(&mut config).await?;
        if config.signing_enabled {
            return Err(anyhow::anyhow!(SynforgeError::BadRequest(
                "disable repository signing before importing a new key".to_string(),
            )));
        }
        let manager = RepoSigningManager;

        manager.remove_all_keys(&config).await?;
        let imported = manager
            .import_private_key(&config, &request.armored_private_key)
            .await?;

        config.signing_key_id = Some(imported.key_id.clone());
        let mut dynamic_settings = BTreeMap::new();
        dynamic_settings.insert(
            "signing_key_id".to_string(),
            serde_json::Value::String(imported.key_id.clone()),
        );
        let armored_private_key = manager
            .export_private_key(&config, imported.key_id.as_str())
            .await?;
        dynamic_settings.insert(
            RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED.to_string(),
            serde_json::Value::String(armored_private_key),
        );
        self.store.upsert_runtime_settings(dynamic_settings).await?;

        if config.signing_enabled {
            manager
                .reconcile_repo_metadata_signature(&config, config.runtime_paths().repo_dir())
                .await?;
        }

        let status = self
            .get_repo_signing_status_for_user_id(Some(current_user_id))
            .await?;
        Ok(ImportRepoSigningKeyResponse {
            key_id: imported.key_id,
            fingerprint: imported.fingerprint,
            status: status.status,
        })
    }

    pub async fn remove_repo_signing_key(
        &self,
        current_user_id: Uuid,
    ) -> anyhow::Result<RepoSigningStatusResponse> {
        let mut config = self.load_effective_daemon_config().await?;
        self.sync_keyring_from_runtime_settings(&mut config).await?;
        if config.signing_enabled {
            return Err(anyhow::anyhow!(SynforgeError::BadRequest(
                "disable repository signing before deleting the key".to_string(),
            )));
        }
        let manager = RepoSigningManager;
        let status = manager.status(&config).await?;
        if !status.key_present {
            return Err(anyhow::anyhow!(SynforgeError::BadRequest(
                "no signing key is present to delete".to_string(),
            )));
        }
        manager.remove_all_keys(&config).await?;

        config.signing_enabled = false;
        config.signing_key_id = None;
        let mut dynamic_settings = BTreeMap::new();
        dynamic_settings.insert(
            "signing_enabled".to_string(),
            serde_json::Value::Bool(false),
        );
        dynamic_settings.insert("signing_key_id".to_string(), serde_json::Value::Null);
        dynamic_settings.insert(
            RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED.to_string(),
            serde_json::Value::Null,
        );
        self.store.upsert_runtime_settings(dynamic_settings).await?;

        manager
            .reconcile_repo_metadata_signature(&config, config.runtime_paths().repo_dir())
            .await?;
        self.get_repo_signing_status_for_user_id(Some(current_user_id))
            .await
    }

    pub async fn test_repo_signing(&self) -> anyhow::Result<TestRepoSigningResponse> {
        let mut config = self.load_effective_daemon_config().await?;
        self.sync_keyring_from_runtime_settings(&mut config).await?;
        let manager = RepoSigningManager;
        if !config.signing_enabled {
            return Err(anyhow::anyhow!(SynforgeError::BadRequest(
                "repository signing is disabled".to_string(),
            )));
        }
        let status = manager.status(&config).await?;
        if !status.key_present || config.signing_key_id.is_none() {
            return Err(anyhow::anyhow!(SynforgeError::BadRequest(
                "no signing key is present; generate or import a key first".to_string(),
            )));
        }
        manager
            .reconcile_repo_metadata_signature(&config, config.runtime_paths().repo_dir())
            .await?;
        let signature_path =
            manager.repo_public_key_path(&config, config.runtime_paths().repo_dir());
        if !tokio::fs::try_exists(&signature_path).await? {
            return Err(anyhow::anyhow!(SynforgeError::Internal(
                "repository public key file was not generated".to_string(),
            )));
        }
        Ok(TestRepoSigningResponse {
            signed: true,
            signature_path,
        })
    }

    pub async fn export_repo_signing_private_key(
        &self,
        current_user: &UserAccount,
    ) -> anyhow::Result<ExportRepoSigningKeyResponse> {
        if !self.is_bootstrap_admin_user(current_user.id).await? {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        }
        let mut config = self.load_effective_daemon_config().await?;
        self.sync_keyring_from_runtime_settings(&mut config).await?;
        let manager = RepoSigningManager;
        let status = manager.status(&config).await?;
        if !status.key_present {
            return Err(anyhow::anyhow!(SynforgeError::BadRequest(
                "no signing key is present; generate or import a key first".to_string(),
            )));
        }
        let key_id = config
            .signing_key_id
            .clone()
            .or(status.active_fingerprint.clone())
            .ok_or_else(|| {
                anyhow::anyhow!(SynforgeError::BadRequest(
                    "no signing key id could be resolved for export".to_string()
                ))
            })?;
        let fingerprint = status.active_fingerprint.unwrap_or_else(|| key_id.clone());
        if config.signing_key_id.as_deref() != Some(key_id.as_str()) {
            config.signing_key_id = Some(key_id.clone());
            let mut settings = BTreeMap::new();
            settings.insert(
                "signing_key_id".to_string(),
                serde_json::Value::String(key_id.clone()),
            );
            self.store.upsert_runtime_settings(settings).await?;
        }
        let armored_private_key = manager.export_private_key(&config, key_id.as_str()).await?;
        let mut settings = BTreeMap::new();
        settings.insert(
            "signing_key_id".to_string(),
            serde_json::Value::String(key_id.clone()),
        );
        settings.insert(
            RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED.to_string(),
            serde_json::Value::String(armored_private_key.clone()),
        );
        self.store.upsert_runtime_settings(settings).await?;
        Ok(ExportRepoSigningKeyResponse {
            key_id,
            fingerprint,
            armored_private_key,
        })
    }

    pub async fn export_repo_signing_public_key(
        &self,
    ) -> anyhow::Result<ExportRepoSigningPublicKeyResponse> {
        let mut config = self.load_effective_daemon_config().await?;
        self.sync_keyring_from_runtime_settings(&mut config).await?;
        let manager = RepoSigningManager;
        let (key_id, fingerprint) = manager.resolve_configured_signing_identity(&config).await?;
        let armored_public_key = manager.export_public_key(&config, key_id.as_str()).await?;
        Ok(ExportRepoSigningPublicKeyResponse {
            key_id,
            fingerprint,
            public_key_name: synforge_core::config::DEFAULT_SIGNING_PUBLIC_KEY_NAME.to_string(),
            armored_public_key,
        })
    }

    async fn get_repo_signing_status_for_user_id(
        &self,
        user_id: Option<Uuid>,
    ) -> anyhow::Result<RepoSigningStatusResponse> {
        let mut config = self.load_effective_daemon_config().await?;
        self.sync_keyring_from_runtime_settings(&mut config).await?;
        let manager = RepoSigningManager;
        let status = manager.status(&config).await?;
        let can_export_private_key = if let Some(user_id) = user_id {
            self.is_bootstrap_admin_user(user_id).await?
        } else {
            false
        };
        Ok(RepoSigningStatusResponse {
            status: RepoSigningStatusView {
                enabled: status.enabled,
                configured_key_id: status.configured_key_id,
                keyring_dir: status.keyring_dir,
                repo_public_key_path: manager
                    .repo_public_key_path(&config, config.runtime_paths().repo_dir()),
                key_present: status.key_present,
                active_fingerprint: status.active_fingerprint,
                can_export_private_key,
            },
        })
    }

    async fn reconcile_existing_artifacts(
        &self,
        config: &DaemonConfig,
        mode: RepoSigningReconcileMode,
    ) -> anyhow::Result<()> {
        let mut config = config.clone();
        self.sync_keyring_from_runtime_settings(&mut config).await?;
        let operation_id = Uuid::now_v7();
        let total_artifacts = self
            .store
            .count_published_repo_files(None, None, None)
            .await?;
        self.update_signing_progress(RepoSigningReconcileProgressView {
            operation_id,
            mode: mode.clone(),
            state: RepoSigningReconcileState::Running,
            total_artifacts,
            processed_artifacts: 0,
            failed_artifacts: 0,
            message: None,
        })
        .await;

        let manager = RepoSigningManager;
        let (key_id, fingerprint) = if mode == RepoSigningReconcileMode::Sign {
            let (key_id, fingerprint) =
                manager.resolve_configured_signing_identity(&config).await?;
            (Some(key_id), Some(fingerprint))
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
                let files = self
                    .store
                    .list_published_repo_files(PAGE_SIZE, offset, None, None, None)
                    .await?;
                if files.is_empty() {
                    break;
                }
                offset += files.len();
                for file in files {
                    processed_artifacts += 1;
                    if !is_signable_existing_artifact(file.kind) {
                        self.update_signing_progress(RepoSigningReconcileProgressView {
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
                    let signature_result = match mode {
                        RepoSigningReconcileMode::Sign => {
                            let Some(key_id) = key_id.as_deref() else {
                                return Err(anyhow::anyhow!("missing signing key id"));
                            };
                            manager
                                .add_signature_to_artifact(&config, key_id, artifact_path.as_path())
                                .await
                        }
                        RepoSigningReconcileMode::Unsign => {
                            manager
                                .remove_signature_from_artifact(artifact_path.as_path())
                                .await
                        }
                    };

                    let signature = match signature_result {
                        Ok(()) => match manager
                            .recompute_artifact_metadata(artifact_path.as_path())
                            .await
                        {
                            Ok((sha256, size_bytes)) => {
                                if let Err(error) = self
                                    .store
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
                                        error_message: Some(
                                            "artifact signature removed".to_string(),
                                        ),
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
                        self.store
                            .upsert_artifact_signatures(std::mem::take(&mut signatures))
                            .await?;
                    }
                    self.update_signing_progress(RepoSigningReconcileProgressView {
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
                self.store.upsert_artifact_signatures(signatures).await?;
            }
            manager
                .reconcile_repo_metadata_signature(&config, config.runtime_paths().repo_dir())
                .await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.update_signing_progress(RepoSigningReconcileProgressView {
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
                self.update_signing_progress(RepoSigningReconcileProgressView {
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

    async fn sync_keyring_from_runtime_settings(
        &self,
        config: &mut DaemonConfig,
    ) -> anyhow::Result<()> {
        let settings = self.store.list_runtime_settings().await?;
        let manager = RepoSigningManager;
        let status = manager.status(config).await?;
        let stored_private_key = settings
            .get(RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);

        let mut updates = BTreeMap::new();

        match (stored_private_key, status.key_present) {
            (Some(armored_private_key), false) => {
                manager.remove_all_keys(config).await?;
                let imported = manager
                    .import_private_key(config, &armored_private_key)
                    .await?;
                if config.signing_key_id.as_deref() != Some(imported.key_id.as_str()) {
                    config.signing_key_id = Some(imported.key_id.clone());
                    updates.insert(
                        "signing_key_id".to_string(),
                        serde_json::Value::String(imported.key_id),
                    );
                }
            }
            _ => {}
        }

        if !updates.is_empty() {
            self.store.upsert_runtime_settings(updates).await?;
        }
        Ok(())
    }

    async fn update_signing_progress(&self, progress: RepoSigningReconcileProgressView) {
        let mut slot = self.signing_reconcile_progress.lock().await;
        *slot = Some(progress);
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
