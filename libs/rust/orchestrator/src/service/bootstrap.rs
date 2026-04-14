use std::collections::BTreeMap;

use serde_json::Value;
use synforge_core::config::DaemonConfig;
use synforge_runtime::RepoSigningManager;
use synforge_store::{DieselStore, UserStore};

use super::config::apply_config_settings;

pub(crate) async fn apply_startup_runtime_overrides(
    store: &DieselStore,
    config: &mut DaemonConfig,
) -> anyhow::Result<()> {
    let settings = store.list_runtime_settings().await?;
    apply_config_settings(config, &settings, true)?;
    let mut updates = BTreeMap::new();
    if !settings.contains_key("session_secret") {
        updates.insert(
            "session_secret".to_string(),
            Value::String(config.session_secret.clone()),
        );
    }
    if !config.bootstrap_completed && store.user_count().await? > 0 {
        config.bootstrap_completed = true;
        updates.insert("bootstrap_completed".to_string(), Value::Bool(true));
    } else if config.bootstrap_completed && !settings.contains_key("bootstrap_completed") {
        updates.insert("bootstrap_completed".to_string(), Value::Bool(true));
    }
    if !updates.is_empty() {
        store.upsert_runtime_settings(updates).await?;
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
        .get("signing_private_key_armored")
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
        store.upsert_runtime_settings(updates).await?;
    }

    Ok(())
}
