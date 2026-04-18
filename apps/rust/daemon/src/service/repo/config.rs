use std::collections::BTreeMap;

use serde_json::Value;
use synforge_core::config::{DaemonConfig, RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED};
use synforge_database::DieselStore;
use synforge_database::runtime_settings::PostgresRuntimeSettingsStore;
use synforge_publish::RepoSigningManager;

pub(crate) async fn sync_keyring_from_runtime_settings(
    store: &DieselStore,
    config: &mut DaemonConfig,
) -> anyhow::Result<()> {
    let runtime_settings = PostgresRuntimeSettingsStore::new(store.clone());
    let settings = runtime_settings.list().await?;
    let manager = RepoSigningManager;
    let status = manager.status(config).await?;
    let stored_private_key = settings
        .get(RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    let mut updates = BTreeMap::new();

    if let (Some(armored_private_key), false) = (stored_private_key, status.key_present) {
        manager.remove_all_keys(config).await?;
        let imported = manager
            .import_private_key(config, &armored_private_key)
            .await?;
        if config.signing_key_id.as_deref() != Some(imported.key_id.as_str()) {
            config.signing_key_id = Some(imported.key_id.clone());
            updates.insert("signing_key_id".to_string(), Value::String(imported.key_id));
        }
    }

    if !updates.is_empty() {
        runtime_settings.upsert(updates).await?;
    }
    Ok(())
}
