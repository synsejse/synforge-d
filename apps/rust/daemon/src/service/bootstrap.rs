use std::collections::BTreeMap;

use serde_json::Value;
use synforge_core::config::{
    DaemonConfig, RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED, apply_config_settings,
};
use synforge_database::DieselStore;
use synforge_database::runtime_settings::PostgresRuntimeSettingsStore;
use synforge_database::users::PostgresUserStore;
use synforge_publish::RepoSigningManager;

pub(crate) async fn apply_startup_runtime_overrides(
    store: &DieselStore,
    config: &mut DaemonConfig,
) -> anyhow::Result<()> {
    let runtime_settings = PostgresRuntimeSettingsStore::new(store.clone());
    let settings = runtime_settings.list().await?;
    apply_config_settings(config, &settings, true)?;
    let mut updates = BTreeMap::new();
    if !config.bootstrap_completed && PostgresUserStore::new(store.clone()).user_count().await? > 0
    {
        config.bootstrap_completed = true;
        updates.insert("bootstrap_completed".to_string(), Value::Bool(true));
    } else if config.bootstrap_completed && !settings.contains_key("bootstrap_completed") {
        updates.insert("bootstrap_completed".to_string(), Value::Bool(true));
    }
    if !updates.is_empty() {
        runtime_settings.upsert(updates).await?;
    }
    config
        .validate()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    if !config.signing_enabled {
        return Ok(());
    }

    let signing_manager = RepoSigningManager;
    let status = signing_manager.status(config).await?;
    if status.key_present {
        return Ok(());
    }

    let Some(armored_private_key) = settings
        .get(RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };

    signing_manager.remove_all_keys(config).await?;
    let imported = signing_manager
        .import_private_key(config, armored_private_key)
        .await?;
    if config.signing_key_id.as_deref() != Some(imported.key_id.as_str()) {
        config.signing_key_id = Some(imported.key_id.clone());
        let mut updates = BTreeMap::new();
        updates.insert("signing_key_id".to_string(), Value::String(imported.key_id));
        runtime_settings.upsert(updates).await?;
    }

    Ok(())
}
