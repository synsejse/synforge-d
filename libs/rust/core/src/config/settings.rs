use std::collections::BTreeMap;

use serde_json::Value;

use super::{DaemonConfig, editable_config_fields};

pub const RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED: &str = "signing_private_key_armored";

pub fn apply_config_settings(
    config: &mut DaemonConfig,
    settings: &BTreeMap<String, Value>,
    allow_internal_runtime_settings: bool,
) -> anyhow::Result<()> {
    validate_config_setting_keys(settings, allow_internal_runtime_settings)?;

    apply_config_setting_values(config, settings)
}

pub fn apply_live_config_settings(
    config: &mut DaemonConfig,
    settings: &BTreeMap<String, Value>,
) -> anyhow::Result<()> {
    validate_config_setting_keys(settings, true)?;
    let live_settings = settings
        .iter()
        .filter(|(key, _)| is_live_runtime_setting_key(key))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    apply_config_setting_values(config, &live_settings)
}

fn validate_config_setting_keys(
    settings: &BTreeMap<String, Value>,
    allow_internal_runtime_settings: bool,
) -> anyhow::Result<()> {
    for key in settings.keys() {
        if is_internal_runtime_setting_key(key) {
            if !allow_internal_runtime_settings {
                anyhow::bail!("config setting is not editable: {key}");
            }
            continue;
        }
        if editable_config_fields()
            .iter()
            .all(|field| field.key.as_str() != key)
            && !is_non_runtime_dynamic_key(key)
        {
            anyhow::bail!("unknown config setting: {key}");
        }
    }
    Ok(())
}

fn apply_config_setting_values(
    config: &mut DaemonConfig,
    settings: &BTreeMap<String, Value>,
) -> anyhow::Result<()> {
    if let Some(value) = settings.get("bootstrap_completed") {
        config.bootstrap_completed = parse_bool_setting(value, "bootstrap_completed")?;
    }

    if let Some(value) = settings.get("listen_addr") {
        config.listen_addr = parse_string_setting(value, "listen_addr")?;
    }
    if let Some(value) = settings.get("public_base_url") {
        config.public_base_url = parse_string_setting(value, "public_base_url")?;
    }
    if let Some(value) = settings.get("worker_image") {
        config.worker_image = parse_string_setting(value, "worker_image")?;
    }
    if let Some(value) = settings.get("signing_enabled") {
        config.signing_enabled = parse_bool_setting(value, "signing_enabled")?;
    }
    if let Some(value) = settings.get("signing_key_id") {
        config.signing_key_id = parse_optional_string_setting(value, "signing_key_id")?;
    }
    if let Some(value) = settings.get("max_concurrent_builds") {
        config.max_concurrent_builds = parse_usize_setting(value, "max_concurrent_builds")?;
    }
    if let Some(value) = settings.get("db_pool_size") {
        config.db_pool_size = parse_u32_setting(value, "db_pool_size")?;
    }
    if let Some(value) = settings.get("queue_buffer_size") {
        config.queue_buffer_size = parse_usize_setting(value, "queue_buffer_size")?;
    }
    if let Some(value) = settings.get("poller_tick_seconds") {
        config.poller_tick_seconds = parse_u64_setting(value, "poller_tick_seconds")?;
    }
    if let Some(value) = settings.get("worker_result_timeout_seconds") {
        config.worker_result_timeout_seconds =
            parse_u64_setting(value, "worker_result_timeout_seconds")?;
    }
    if let Some(value) = settings.get("worker_socket_timeout_seconds") {
        config.worker_socket_timeout_seconds =
            parse_u64_setting(value, "worker_socket_timeout_seconds")?;
    }
    if let Some(value) = settings.get("git_operation_timeout_seconds") {
        config.git_operation_timeout_seconds =
            parse_u64_setting(value, "git_operation_timeout_seconds")?;
    }
    if let Some(value) = settings.get("mock_chroot_cache_ttl_seconds") {
        config.mock_chroot_cache_ttl_seconds =
            parse_u64_setting(value, "mock_chroot_cache_ttl_seconds")?;
    }
    if let Some(value) = settings.get("git_mirror_refresh_ttl_seconds") {
        config.git_mirror_refresh_ttl_seconds =
            parse_u64_setting(value, "git_mirror_refresh_ttl_seconds")?;
    }
    if let Some(value) = settings.get("git_mirror_max_unused_seconds") {
        config.git_mirror_max_unused_seconds =
            parse_u64_setting(value, "git_mirror_max_unused_seconds")?;
    }
    if let Some(value) = settings.get("build_failure_backoff_base_seconds") {
        config.build_failure_backoff_base_seconds =
            parse_u64_setting(value, "build_failure_backoff_base_seconds")?;
    }
    if let Some(value) = settings.get("build_failure_backoff_max_seconds") {
        config.build_failure_backoff_max_seconds =
            parse_u64_setting(value, "build_failure_backoff_max_seconds")?;
    }
    Ok(())
}

pub fn ensure_runtime_editable_keys(settings: &BTreeMap<String, Value>) -> anyhow::Result<()> {
    for key in settings.keys() {
        let Some(field) = editable_config_fields()
            .into_iter()
            .find(|field| field.key == *key)
        else {
            anyhow::bail!("unknown config setting: {key}");
        };
        if !field.editable_in_runtime {
            anyhow::bail!("config setting is not editable at runtime: {key}");
        }
    }
    Ok(())
}

pub fn ensure_setup_editable_keys(settings: &BTreeMap<String, Value>) -> anyhow::Result<()> {
    for key in settings.keys() {
        let Some(field) = editable_config_fields()
            .into_iter()
            .find(|field| field.key == *key)
        else {
            anyhow::bail!("unknown config setting: {key}");
        };
        if !field.editable_in_setup {
            anyhow::bail!("config setting is not editable during setup: {key}");
        }
    }
    Ok(())
}

pub fn daemon_config_runtime_settings(config: &DaemonConfig) -> BTreeMap<String, Value> {
    let mut settings = BTreeMap::new();
    settings.insert(
        "listen_addr".to_string(),
        Value::String(config.listen_addr.clone()),
    );
    settings.insert(
        "public_base_url".to_string(),
        Value::String(config.public_base_url.clone()),
    );
    settings.insert(
        "worker_image".to_string(),
        Value::String(config.worker_image.clone()),
    );
    settings.insert(
        "signing_enabled".to_string(),
        Value::Bool(config.signing_enabled),
    );
    settings.insert(
        "signing_key_id".to_string(),
        config
            .signing_key_id
            .as_ref()
            .map(|value| Value::String(value.clone()))
            .unwrap_or(Value::Null),
    );
    settings.insert(
        "max_concurrent_builds".to_string(),
        Value::from(config.max_concurrent_builds as u64),
    );
    settings.insert(
        "db_pool_size".to_string(),
        Value::from(config.db_pool_size as u64),
    );
    settings.insert(
        "queue_buffer_size".to_string(),
        Value::from(config.queue_buffer_size as u64),
    );
    settings.insert(
        "poller_tick_seconds".to_string(),
        Value::from(config.poller_tick_seconds),
    );
    settings.insert(
        "worker_result_timeout_seconds".to_string(),
        Value::from(config.worker_result_timeout_seconds),
    );
    settings.insert(
        "worker_socket_timeout_seconds".to_string(),
        Value::from(config.worker_socket_timeout_seconds),
    );
    settings.insert(
        "git_operation_timeout_seconds".to_string(),
        Value::from(config.git_operation_timeout_seconds),
    );
    settings.insert(
        "mock_chroot_cache_ttl_seconds".to_string(),
        Value::from(config.mock_chroot_cache_ttl_seconds),
    );
    settings.insert(
        "git_mirror_refresh_ttl_seconds".to_string(),
        Value::from(config.git_mirror_refresh_ttl_seconds),
    );
    settings.insert(
        "git_mirror_max_unused_seconds".to_string(),
        Value::from(config.git_mirror_max_unused_seconds),
    );
    settings.insert(
        "build_failure_backoff_base_seconds".to_string(),
        Value::from(config.build_failure_backoff_base_seconds),
    );
    settings.insert(
        "build_failure_backoff_max_seconds".to_string(),
        Value::from(config.build_failure_backoff_max_seconds),
    );
    settings.insert(
        "bootstrap_completed".to_string(),
        Value::Bool(config.bootstrap_completed),
    );
    settings
}

pub fn pending_restart_config_settings(
    config: &DaemonConfig,
    stored_settings: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    let active_settings = daemon_config_runtime_settings(config);
    editable_config_fields()
        .into_iter()
        .filter(|field| field.restart_required)
        .filter_map(|field| {
            let stored = stored_settings.get(&field.key)?;
            (active_settings.get(&field.key) != Some(stored)).then(|| (field.key, stored.clone()))
        })
        .collect()
}

fn is_non_runtime_dynamic_key(key: &str) -> bool {
    matches!(key, "signing_enabled" | "signing_key_id")
}

fn is_live_runtime_setting_key(key: &str) -> bool {
    matches!(
        key,
        "bootstrap_completed" | "signing_enabled" | "signing_key_id"
    )
}

fn is_internal_runtime_setting_key(key: &str) -> bool {
    key == RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED
        || key == "database_url"
        || key == "session_secret"
        || key == "bootstrap_completed"
}

fn parse_string_setting(value: &Value, key: &str) -> anyhow::Result<String> {
    let Some(value) = value.as_str() else {
        anyhow::bail!("config setting must be a string: {key}");
    };
    let value = value.trim().to_string();
    if value.is_empty() {
        anyhow::bail!("config setting must not be empty: {key}");
    }
    Ok(value)
}

fn parse_optional_string_setting(value: &Value, key: &str) -> anyhow::Result<Option<String>> {
    if value.is_null() {
        return Ok(None);
    }
    Ok(Some(parse_string_setting(value, key)?))
}

fn parse_bool_setting(value: &Value, key: &str) -> anyhow::Result<bool> {
    let Some(value) = value.as_bool() else {
        anyhow::bail!("config setting must be a boolean: {key}");
    };
    Ok(value)
}

fn parse_u64_setting(value: &Value, key: &str) -> anyhow::Result<u64> {
    let Some(value) = value.as_u64() else {
        anyhow::bail!("config setting must be a positive integer: {key}");
    };
    if value == 0 {
        anyhow::bail!("config setting must be greater than zero: {key}");
    }
    Ok(value)
}

fn parse_usize_setting(value: &Value, key: &str) -> anyhow::Result<usize> {
    Ok(parse_u64_setting(value, key)? as usize)
}

fn parse_u32_setting(value: &Value, key: &str) -> anyhow::Result<u32> {
    u32::try_from(parse_u64_setting(value, key)?)
        .map_err(|_| anyhow::anyhow!("config setting is out of range: {key}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_settings_do_not_change_restart_required_values() {
        let mut config = DaemonConfig::default();
        let original_worker_image = config.worker_image.clone();
        let settings = BTreeMap::from([
            (
                "worker_image".to_string(),
                Value::String("replacement:latest".to_string()),
            ),
            ("signing_enabled".to_string(), Value::Bool(true)),
        ]);

        apply_live_config_settings(&mut config, &settings).expect("apply live settings");

        assert_eq!(config.worker_image, original_worker_image);
        assert!(config.signing_enabled);
    }

    #[test]
    fn pending_restart_settings_only_include_changed_restart_fields() {
        let config = DaemonConfig::default();
        let settings = BTreeMap::from([
            (
                "worker_image".to_string(),
                Value::String("replacement:latest".to_string()),
            ),
            (
                "public_base_url".to_string(),
                Value::String(config.public_base_url.clone()),
            ),
            ("signing_enabled".to_string(), Value::Bool(true)),
        ]);

        let pending = pending_restart_config_settings(&config, &settings);

        assert_eq!(
            pending,
            BTreeMap::from([(
                "worker_image".to_string(),
                Value::String("replacement:latest".to_string())
            )])
        );
    }

    #[test]
    fn generic_runtime_fields_are_explicitly_restart_required() {
        assert!(
            editable_config_fields()
                .into_iter()
                .filter(|field| field.editable_in_runtime)
                .all(|field| field.restart_required)
        );
    }
}
