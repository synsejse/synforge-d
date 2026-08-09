use std::collections::BTreeMap;

use serde_json::Value;
use synforge_core::config::DaemonConfig;
use synforge_publish::RepoSigningManager;

use super::JobLifecycle;

impl JobLifecycle {
    pub(super) async fn load_runtime_overrides(&self) -> anyhow::Result<DaemonConfig> {
        let mut config = self.config.clone();
        let settings = self.store.list_runtime_settings().await?;
        apply_bool_setting(
            &mut config.signing_enabled,
            settings.get("signing_enabled"),
            "signing_enabled",
        )?;
        apply_optional_string_setting(
            &mut config.signing_key_id,
            settings.get("signing_key_id"),
            "signing_key_id",
        )?;
        if config.signing_enabled {
            self.ensure_signing_key_from_runtime_settings(&mut config, &settings)
                .await?;
        }
        Ok(config)
    }

    async fn ensure_signing_key_from_runtime_settings(
        &self,
        config: &mut DaemonConfig,
        settings: &BTreeMap<String, Value>,
    ) -> anyhow::Result<()> {
        let Some(armored_private_key) = settings
            .get("signing_private_key_armored")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Ok(());
        };
        let signing_manager = RepoSigningManager;
        let status = signing_manager.status(config).await?;
        if status.key_present {
            return Ok(());
        }
        signing_manager.remove_all_keys(config).await?;
        let imported = signing_manager
            .import_private_key(config, armored_private_key)
            .await?;
        if config.signing_key_id.as_deref() != Some(imported.key_id.as_str()) {
            config.signing_key_id = Some(imported.key_id.clone());
            let updates =
                BTreeMap::from([("signing_key_id".to_string(), Value::String(imported.key_id))]);
            self.store.upsert_runtime_settings(updates).await?;
        }
        Ok(())
    }
}

fn apply_bool_setting(target: &mut bool, value: Option<&Value>, key: &str) -> anyhow::Result<()> {
    if let Some(value) = value {
        let Some(value) = value.as_bool() else {
            anyhow::bail!("runtime setting must be a boolean: {key}");
        };
        *target = value;
    }
    Ok(())
}

fn apply_optional_string_setting(
    target: &mut Option<String>,
    value: Option<&Value>,
    key: &str,
) -> anyhow::Result<()> {
    if let Some(value) = value {
        if value.is_null() {
            *target = None;
        } else {
            let Some(value) = value.as_str() else {
                anyhow::bail!("runtime setting must be a string or null: {key}");
            };
            let value = value.trim();
            if value.is_empty() {
                anyhow::bail!("runtime setting must not be empty when provided: {key}");
            }
            *target = Some(value.to_string());
        }
    }
    Ok(())
}
