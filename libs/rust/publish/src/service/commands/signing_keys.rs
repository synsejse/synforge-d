use synforge_core::{
    api::{
        ExportRepoSigningKeyResponse, ExportRepoSigningPublicKeyResponse,
        GenerateRepoSigningKeyResponse, ImportRepoSigningKeyRequest, ImportRepoSigningKeyResponse,
        RepoSigningReconcileMode, RepoSigningStatusResponse, TestRepoSigningResponse,
        UpdateRepoSigningConfigRequest,
    },
    constants::DEFAULT_SIGNING_PUBLIC_KEY_NAME,
    error::SynforgeError,
};

use super::{
    RepoArtifactCatalog, RepoSigningCommandRunner, RepoSigningProgressWriter,
    RepoSigningSettingsWriter, signing_reconcile::reconcile_existing_artifacts,
};
use crate::service::queries::{
    RepoFileStorage, RepoSigningConfigLoader, RepoSigningInspector, get_repo_signing_status,
};
use crate::service::state::RepoSigningSettingsUpdate;

pub async fn update_repo_signing_config<D>(
    deps: &D,
    request: UpdateRepoSigningConfigRequest,
    can_export_private_key: bool,
) -> anyhow::Result<RepoSigningStatusResponse>
where
    D: RepoSigningConfigLoader
        + RepoSigningInspector
        + RepoSigningSettingsWriter
        + RepoSigningCommandRunner
        + RepoArtifactCatalog
        + RepoSigningProgressWriter
        + RepoFileStorage
        + Send
        + Sync,
{
    let mut config = deps.load_effective_daemon_config().await?;
    deps.sync_keyring_from_runtime_settings(&mut config).await?;
    config.signing_enabled = request.enabled;

    if config.signing_enabled && config.signing_key_id.is_none() {
        return Err(anyhow::anyhow!(SynforgeError::BadRequest(
            "generate or import a private key before enabling repository signing".to_string(),
        )));
    }
    if config.signing_enabled {
        let status = deps.inspect_repo_signing_status(&config).await?;
        if !status.key_present {
            return Err(anyhow::anyhow!(SynforgeError::BadRequest(
                "no private key material is available; generate or import a key first".to_string(),
            )));
        }
    }

    deps.persist_repo_signing_settings(RepoSigningSettingsUpdate {
        signing_enabled: Some(config.signing_enabled),
        ..RepoSigningSettingsUpdate::default()
    })
    .await?;

    let mode = if config.signing_enabled {
        RepoSigningReconcileMode::Sign
    } else {
        RepoSigningReconcileMode::Unsign
    };
    reconcile_existing_artifacts(deps, &config, mode).await?;
    get_repo_signing_status(deps, can_export_private_key).await
}

pub async fn generate_repo_signing_key<D>(
    deps: &D,
    can_export_private_key: bool,
) -> anyhow::Result<GenerateRepoSigningKeyResponse>
where
    D: RepoSigningConfigLoader
        + RepoSigningInspector
        + RepoSigningSettingsWriter
        + RepoSigningCommandRunner
        + Send
        + Sync,
{
    let mut config = deps.load_effective_daemon_config().await?;
    deps.sync_keyring_from_runtime_settings(&mut config).await?;
    if config.signing_enabled {
        return Err(anyhow::anyhow!(SynforgeError::BadRequest(
            "disable repository signing before generating a new key".to_string(),
        )));
    }

    let managed = deps.generate_new_managed_signing_key(&config).await?;
    config.signing_key_id = Some(managed.key_id.clone());
    let armored_private_key = deps
        .export_private_key(&config, managed.key_id.as_str())
        .await?;
    deps.persist_repo_signing_settings(RepoSigningSettingsUpdate {
        signing_key_id: Some(Some(managed.key_id.clone())),
        armored_private_key: Some(Some(armored_private_key)),
        ..RepoSigningSettingsUpdate::default()
    })
    .await?;

    let status = get_repo_signing_status(deps, can_export_private_key).await?;
    Ok(GenerateRepoSigningKeyResponse {
        key_id: managed.key_id,
        fingerprint: managed.fingerprint,
        status: status.status,
    })
}

pub async fn import_repo_signing_key<D>(
    deps: &D,
    request: ImportRepoSigningKeyRequest,
    can_export_private_key: bool,
) -> anyhow::Result<ImportRepoSigningKeyResponse>
where
    D: RepoSigningConfigLoader
        + RepoSigningInspector
        + RepoSigningSettingsWriter
        + RepoSigningCommandRunner
        + Send
        + Sync,
{
    let mut config = deps.load_effective_daemon_config().await?;
    deps.sync_keyring_from_runtime_settings(&mut config).await?;
    if config.signing_enabled {
        return Err(anyhow::anyhow!(SynforgeError::BadRequest(
            "disable repository signing before importing a new key".to_string(),
        )));
    }

    deps.remove_all_keys(&config).await?;
    let imported = deps
        .import_private_key(&config, &request.armored_private_key)
        .await?;
    config.signing_key_id = Some(imported.key_id.clone());
    let armored_private_key = deps
        .export_private_key(&config, imported.key_id.as_str())
        .await?;
    deps.persist_repo_signing_settings(RepoSigningSettingsUpdate {
        signing_key_id: Some(Some(imported.key_id.clone())),
        armored_private_key: Some(Some(armored_private_key)),
        ..RepoSigningSettingsUpdate::default()
    })
    .await?;

    let status = get_repo_signing_status(deps, can_export_private_key).await?;
    Ok(ImportRepoSigningKeyResponse {
        key_id: imported.key_id,
        fingerprint: imported.fingerprint,
        status: status.status,
    })
}

pub async fn remove_repo_signing_key<D>(
    deps: &D,
    can_export_private_key: bool,
) -> anyhow::Result<RepoSigningStatusResponse>
where
    D: RepoSigningConfigLoader
        + RepoSigningInspector
        + RepoSigningSettingsWriter
        + RepoSigningCommandRunner
        + RepoArtifactCatalog
        + Send
        + Sync,
{
    let mut config = deps.load_effective_daemon_config().await?;
    deps.sync_keyring_from_runtime_settings(&mut config).await?;
    if config.signing_enabled {
        return Err(anyhow::anyhow!(SynforgeError::BadRequest(
            "disable repository signing before deleting the key".to_string(),
        )));
    }

    let status = deps.inspect_repo_signing_status(&config).await?;
    if !status.key_present {
        return Err(anyhow::anyhow!(SynforgeError::BadRequest(
            "no signing key is present to delete".to_string(),
        )));
    }

    deps.remove_all_keys(&config).await?;
    config.signing_enabled = false;
    config.signing_key_id = None;
    deps.persist_repo_signing_settings(RepoSigningSettingsUpdate {
        signing_enabled: Some(false),
        signing_key_id: Some(None),
        armored_private_key: Some(None),
    })
    .await?;
    deps.reconcile_repo_metadata_signature(&config).await?;
    deps.sync_repo_tree(config.runtime_paths().repo_dir())
        .await?;
    get_repo_signing_status(deps, can_export_private_key).await
}

pub async fn test_repo_signing<D>(deps: &D) -> anyhow::Result<TestRepoSigningResponse>
where
    D: RepoSigningConfigLoader
        + RepoSigningInspector
        + RepoSigningCommandRunner
        + RepoArtifactCatalog
        + Send
        + Sync,
{
    let mut config = deps.load_effective_daemon_config().await?;
    deps.sync_keyring_from_runtime_settings(&mut config).await?;
    if !config.signing_enabled {
        return Err(anyhow::anyhow!(SynforgeError::BadRequest(
            "repository signing is disabled".to_string(),
        )));
    }

    let status = deps.inspect_repo_signing_status(&config).await?;
    if !status.key_present || config.signing_key_id.is_none() {
        return Err(anyhow::anyhow!(SynforgeError::BadRequest(
            "no signing key is present; generate or import a key first".to_string(),
        )));
    }

    deps.reconcile_repo_metadata_signature(&config).await?;
    deps.sync_repo_tree(config.runtime_paths().repo_dir())
        .await?;
    if !tokio::fs::try_exists(&status.repo_public_key_path).await? {
        return Err(anyhow::anyhow!(SynforgeError::Internal(
            "repository public key file was not generated".to_string(),
        )));
    }
    Ok(TestRepoSigningResponse {
        signed: true,
        signature_path: status.repo_public_key_path,
    })
}

pub async fn export_repo_signing_private_key<D>(
    deps: &D,
    authorized: bool,
) -> anyhow::Result<ExportRepoSigningKeyResponse>
where
    D: RepoSigningConfigLoader
        + RepoSigningInspector
        + RepoSigningSettingsWriter
        + RepoSigningCommandRunner
        + Send
        + Sync,
{
    if !authorized {
        return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
    }

    let mut config = deps.load_effective_daemon_config().await?;
    deps.sync_keyring_from_runtime_settings(&mut config).await?;
    let status = deps.inspect_repo_signing_status(&config).await?;
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
                "no signing key id could be resolved for export".to_string(),
            ))
        })?;
    let fingerprint = status.active_fingerprint.unwrap_or_else(|| key_id.clone());
    if config.signing_key_id.as_deref() != Some(key_id.as_str()) {
        config.signing_key_id = Some(key_id.clone());
        deps.persist_repo_signing_settings(RepoSigningSettingsUpdate {
            signing_key_id: Some(Some(key_id.clone())),
            ..RepoSigningSettingsUpdate::default()
        })
        .await?;
    }

    let armored_private_key = deps.export_private_key(&config, key_id.as_str()).await?;
    deps.persist_repo_signing_settings(RepoSigningSettingsUpdate {
        signing_key_id: Some(Some(key_id.clone())),
        armored_private_key: Some(Some(armored_private_key.clone())),
        ..RepoSigningSettingsUpdate::default()
    })
    .await?;
    Ok(ExportRepoSigningKeyResponse {
        key_id,
        fingerprint,
        armored_private_key,
    })
}

pub async fn export_repo_signing_public_key<D>(
    deps: &D,
) -> anyhow::Result<ExportRepoSigningPublicKeyResponse>
where
    D: RepoSigningConfigLoader + RepoSigningCommandRunner + Send + Sync,
{
    let mut config = deps.load_effective_daemon_config().await?;
    deps.sync_keyring_from_runtime_settings(&mut config).await?;
    let identity = deps.resolve_configured_signing_identity(&config).await?;
    let armored_public_key = deps
        .export_public_key(&config, identity.key_id.as_str())
        .await?;
    Ok(ExportRepoSigningPublicKeyResponse {
        key_id: identity.key_id,
        fingerprint: identity.fingerprint,
        public_key_name: DEFAULT_SIGNING_PUBLIC_KEY_NAME.to_string(),
        armored_public_key,
    })
}
