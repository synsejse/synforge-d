use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::Value;
use synforge_core::{
    api::{
        ConfigFieldDescriptor, ConfigFieldType, ConfigSchemaResponse, EffectiveConfigDto,
        EffectiveConfigView, SetupInitializeRequest, UpdateRuntimeSettingsRequest,
    },
    config::DaemonConfig,
};

use super::SynforgeService;
use super::signing::RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED;
use crate::RepoSigningManager;
use crate::db::UserStore;

impl SynforgeService {
    pub async fn config_schema(&self) -> ConfigSchemaResponse {
        ConfigSchemaResponse {
            fields: editable_config_fields(),
        }
    }

    pub(crate) async fn load_effective_daemon_config(&self) -> anyhow::Result<DaemonConfig> {
        let mut current = self.config.clone();
        let dynamic_settings = self.store.list_runtime_settings().await?;
        apply_config_settings(&mut current, &dynamic_settings, true)?;
        let mut updates = BTreeMap::new();
        if !dynamic_settings.contains_key("session_secret") {
            updates.insert(
                "session_secret".to_string(),
                Value::String(current.session_secret.clone()),
            );
        }
        if !current.bootstrap_completed && self.store.user_count().await? > 0 {
            current.bootstrap_completed = true;
            updates.insert("bootstrap_completed".to_string(), Value::Bool(true));
        } else if current.bootstrap_completed
            && !dynamic_settings.contains_key("bootstrap_completed")
        {
            updates.insert("bootstrap_completed".to_string(), Value::Bool(true));
        }
        if !updates.is_empty() {
            self.store.upsert_runtime_settings(updates).await?;
        }
        current
            .validate()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        Ok(current)
    }

    pub async fn effective_config(&self) -> anyhow::Result<EffectiveConfigDto> {
        let current = self.load_effective_daemon_config().await?;
        let paths = current.runtime_paths();
        Ok(EffectiveConfigDto {
            config: EffectiveConfigView {
                bootstrap_completed: current.bootstrap_completed,
                listen_addr: current.listen_addr.clone(),
                runtime_root: current.runtime_root.clone(),
                database_url: current.database_url.clone(),
                packages_dir: paths.packages_dir().to_path_buf(),
                repo_dir: paths.repo_dir().to_path_buf(),
                jobs_root: paths.jobs_root().to_path_buf(),
                worker_image: current.worker_image.clone(),
                worker_jobs_root: current.worker_jobs_root.clone(),
                signing_enabled: current.signing_enabled,
                signing_key_id: current.signing_key_id.clone(),
                max_concurrent_builds: current.max_concurrent_builds,
                db_pool_size: current.db_pool_size,
                queue_buffer_size: current.queue_buffer_size,
                poller_tick_seconds: current.poller_tick_seconds,
                worker_result_timeout_seconds: current.worker_result_timeout_seconds,
                worker_socket_timeout_seconds: current.worker_socket_timeout_seconds,
                git_operation_timeout_seconds: current.git_operation_timeout_seconds,
                public_base_url: current.public_base_url,
                mock_chroot_cache_ttl_seconds: current.mock_chroot_cache_ttl_seconds,
                git_mirror_refresh_ttl_seconds: current.git_mirror_refresh_ttl_seconds,
                git_mirror_max_unused_seconds: current.git_mirror_max_unused_seconds,
            },
        })
    }

    pub async fn update_runtime_settings(
        &self,
        request: UpdateRuntimeSettingsRequest,
    ) -> anyhow::Result<EffectiveConfigDto> {
        let request_settings = request.settings;
        ensure_runtime_editable_keys(&request_settings)?;

        let mut config = self.load_effective_daemon_config().await?;
        let mut merged_settings = self.store.list_runtime_settings().await?;
        for (key, value) in request_settings.clone() {
            merged_settings.insert(key, value);
        }
        apply_config_settings(&mut config, &merged_settings, true)?;
        config
            .validate()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;

        self.store.upsert_runtime_settings(request_settings).await?;
        self.effective_config().await
    }

    pub async fn initialize_setup(
        &self,
        request: SetupInitializeRequest,
    ) -> anyhow::Result<EffectiveConfigDto> {
        let SetupInitializeRequest {
            settings,
            admin,
            enable_signing,
            signing_armored_private_key,
        } = request;
        let current = self.load_effective_daemon_config().await?;
        if current.bootstrap_completed {
            anyhow::bail!("setup has already been completed");
        }
        super::validate_user_handle(&admin.handle)?;
        super::validate_display_name(&admin.display_name)?;
        super::validate_password(&admin.password)?;
        let request_settings = settings;
        ensure_setup_editable_keys(&request_settings)?;
        let mut config = current;
        apply_config_settings(&mut config, &request_settings, false)?;
        config.bootstrap_completed = false;
        config
            .validate()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        if self.store.user_count().await? > 0 {
            anyhow::bail!("initial admin cannot be created because users already exist");
        }
        self.bootstrap_admin(&admin.handle, &admin.display_name, &admin.password)
            .await?;
        let mut runtime_settings = daemon_config_runtime_settings(&config);
        let signing_manager = RepoSigningManager;
        if enable_signing {
            let managed_key = if let Some(armored_private_key) = signing_armored_private_key
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                signing_manager.remove_all_keys(&config).await?;
                signing_manager
                    .import_private_key(&config, armored_private_key)
                    .await?
            } else {
                signing_manager.ensure_managed_signing_key(&config).await?
            };
            let armored_private_key = signing_manager
                .export_private_key(&config, managed_key.key_id.as_str())
                .await?;
            runtime_settings.insert(
                "signing_key_id".to_string(),
                Value::String(managed_key.key_id.clone()),
            );
            runtime_settings.insert("signing_enabled".to_string(), Value::Bool(true));
            runtime_settings.insert(
                RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED.to_string(),
                Value::String(armored_private_key),
            );
            config.signing_enabled = true;
            config.signing_key_id = Some(managed_key.key_id);
        } else {
            runtime_settings.insert("signing_enabled".to_string(), Value::Bool(false));
            runtime_settings.insert("signing_key_id".to_string(), Value::Null);
            runtime_settings.insert(
                RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED.to_string(),
                Value::Null,
            );
            config.signing_enabled = false;
            config.signing_key_id = None;
        }
        config.bootstrap_completed = true;
        runtime_settings.insert("bootstrap_completed".to_string(), Value::Bool(true));
        self.store.upsert_runtime_settings(runtime_settings).await?;
        signing_manager
            .reconcile_repo_metadata_signature(&config, config.runtime_paths().repo_dir())
            .await?;
        self.effective_config().await
    }
}

#[derive(Copy, Clone)]
struct ConfigSection<'a> {
    key: &'a str,
    label: &'a str,
}

#[derive(Copy, Clone)]
struct ConfigEditability {
    in_setup: bool,
    in_runtime: bool,
}

fn editable_config_fields() -> Vec<ConfigFieldDescriptor> {
    vec![
        config_string_field(
            ConfigSection {
                key: "server",
                label: "Server",
            },
            "listen_addr",
            "Listen address",
            "Daemon HTTP listen address.",
            "0.0.0.0:8080",
            ConfigEditability {
                in_setup: true,
                in_runtime: false,
            },
        ),
        config_string_field(
            ConfigSection {
                key: "storage",
                label: "Storage",
            },
            "runtime_root",
            "Runtime root",
            "Root directory for database, package metadata, repo files, and jobs.",
            "/var/lib/synforge",
            ConfigEditability {
                in_setup: true,
                in_runtime: false,
            },
        ),
        config_string_field(
            ConfigSection {
                key: "server",
                label: "Server",
            },
            "public_base_url",
            "Public base URL",
            "Base URL used in generated links and repo setup.",
            "http://localhost:8080",
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_string_field(
            ConfigSection {
                key: "worker",
                label: "Worker",
            },
            "worker_image",
            "Worker image",
            "Docker image used for spawned worker containers.",
            "synforge-worker-fedora:latest",
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_optional_string_field(
            ConfigSection {
                key: "worker",
                label: "Worker",
            },
            "worker_jobs_root",
            "Worker jobs root",
            "Host path used for per-job worker mock bind mounts.",
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_bool_field(
            ConfigSection {
                key: "signing",
                label: "Signing",
            },
            "signing_enabled",
            "Repository GPG signing enabled",
            "Enable package signing for repository artifacts.",
            false,
            ConfigEditability {
                in_setup: false,
                in_runtime: false,
            },
        ),
        config_optional_string_field(
            ConfigSection {
                key: "signing",
                label: "Signing",
            },
            "signing_key_id",
            "Repository GPG key ID",
            "Configured signing key identifier. Managed by the dedicated signing APIs.",
            ConfigEditability {
                in_setup: false,
                in_runtime: false,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "build",
                label: "Build",
            },
            "max_concurrent_builds",
            "Max concurrent builds",
            "Maximum number of active builds at once.",
            2,
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "database",
                label: "Database",
            },
            "db_pool_size",
            "DB pool size",
            "Number of database connection pool slots.",
            5,
            ConfigEditability {
                in_setup: true,
                in_runtime: false,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "scheduler",
                label: "Scheduler",
            },
            "queue_buffer_size",
            "Queue buffer size",
            "In-memory queued build channel capacity.",
            128,
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "scheduler",
                label: "Scheduler",
            },
            "poller_tick_seconds",
            "Poller tick seconds",
            "How often package polling wakes up.",
            30,
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "worker",
                label: "Worker",
            },
            "worker_result_timeout_seconds",
            "Worker result timeout seconds",
            "Timeout while waiting for worker completion after request dispatch.",
            10,
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "worker",
                label: "Worker",
            },
            "worker_socket_timeout_seconds",
            "Worker socket timeout seconds",
            "Socket timeout used for worker protocol I/O.",
            30,
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "git",
                label: "Git",
            },
            "git_operation_timeout_seconds",
            "Git operation timeout seconds",
            "Timeout applied to git inspection and sync commands.",
            600,
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "cache",
                label: "Cache",
            },
            "mock_chroot_cache_ttl_seconds",
            "Mock chroot cache TTL seconds",
            "How long to cache discovered mock chroots before refreshing.",
            300,
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "cache",
                label: "Cache",
            },
            "git_mirror_refresh_ttl_seconds",
            "Git mirror refresh TTL seconds",
            "Maximum age before a cached git mirror is refreshed from origin.",
            300,
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
        config_number_field(
            ConfigSection {
                key: "cache",
                label: "Cache",
            },
            "git_mirror_max_unused_seconds",
            "Git mirror max unused seconds",
            "Remove cached git mirrors that have not been used within this window.",
            604_800,
            ConfigEditability {
                in_setup: true,
                in_runtime: true,
            },
        ),
    ]
}

fn config_string_field(
    section: ConfigSection<'_>,
    key: &str,
    label: &str,
    description: &str,
    default_value: &str,
    editability: ConfigEditability,
) -> ConfigFieldDescriptor {
    ConfigFieldDescriptor {
        key: key.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        section_key: section.key.to_string(),
        section_label: section.label.to_string(),
        field_type: ConfigFieldType::String,
        required: true,
        min_value: None,
        editable_in_setup: editability.in_setup,
        editable_in_runtime: editability.in_runtime,
        default_value: Value::String(default_value.to_string()),
    }
}

fn config_optional_string_field(
    section: ConfigSection<'_>,
    key: &str,
    label: &str,
    description: &str,
    editability: ConfigEditability,
) -> ConfigFieldDescriptor {
    ConfigFieldDescriptor {
        key: key.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        section_key: section.key.to_string(),
        section_label: section.label.to_string(),
        field_type: ConfigFieldType::String,
        required: false,
        min_value: None,
        editable_in_setup: editability.in_setup,
        editable_in_runtime: editability.in_runtime,
        default_value: Value::Null,
    }
}

fn config_number_field(
    section: ConfigSection<'_>,
    key: &str,
    label: &str,
    description: &str,
    default_value: u64,
    editability: ConfigEditability,
) -> ConfigFieldDescriptor {
    ConfigFieldDescriptor {
        key: key.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        section_key: section.key.to_string(),
        section_label: section.label.to_string(),
        field_type: ConfigFieldType::Number,
        required: true,
        min_value: Some(1),
        editable_in_setup: editability.in_setup,
        editable_in_runtime: editability.in_runtime,
        default_value: Value::Number(default_value.into()),
    }
}

fn config_bool_field(
    section: ConfigSection<'_>,
    key: &str,
    label: &str,
    description: &str,
    default_value: bool,
    editability: ConfigEditability,
) -> ConfigFieldDescriptor {
    ConfigFieldDescriptor {
        key: key.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        section_key: section.key.to_string(),
        section_label: section.label.to_string(),
        field_type: ConfigFieldType::Boolean,
        required: true,
        min_value: None,
        editable_in_setup: editability.in_setup,
        editable_in_runtime: editability.in_runtime,
        default_value: Value::Bool(default_value),
    }
}

pub(super) fn apply_config_settings(
    config: &mut DaemonConfig,
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

    if let Some(value) = settings.get("bootstrap_completed") {
        if !allow_internal_runtime_settings {
            anyhow::bail!("config setting is not editable at runtime: bootstrap_completed");
        }
        config.bootstrap_completed = parse_bool_setting(value, "bootstrap_completed")?;
    }

    if let Some(value) = settings.get("listen_addr") {
        config.listen_addr = parse_string_setting(value, "listen_addr")?;
    }
    if let Some(value) = settings.get("runtime_root") {
        config.runtime_root = PathBuf::from(parse_string_setting(value, "runtime_root")?);
    }
    if let Some(value) = settings.get("public_base_url") {
        config.public_base_url = parse_string_setting(value, "public_base_url")?;
    }
    if let Some(value) = settings.get("worker_image") {
        config.worker_image = parse_string_setting(value, "worker_image")?;
    }
    if let Some(value) = settings.get("worker_jobs_root") {
        config.worker_jobs_root =
            parse_optional_string_setting(value, "worker_jobs_root")?.map(PathBuf::from);
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
    if let Some(value) = settings.get("session_secret") {
        if !allow_internal_runtime_settings {
            anyhow::bail!("config setting is not editable at runtime: session_secret");
        }
        config.session_secret = parse_string_setting(value, "session_secret")?;
    }

    Ok(())
}

fn ensure_runtime_editable_keys(settings: &BTreeMap<String, Value>) -> anyhow::Result<()> {
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

fn ensure_setup_editable_keys(settings: &BTreeMap<String, Value>) -> anyhow::Result<()> {
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

fn daemon_config_runtime_settings(config: &DaemonConfig) -> BTreeMap<String, Value> {
    let mut settings = BTreeMap::new();
    settings.insert(
        "listen_addr".to_string(),
        Value::String(config.listen_addr.clone()),
    );
    settings.insert(
        "runtime_root".to_string(),
        Value::String(config.runtime_root.display().to_string()),
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
        "worker_jobs_root".to_string(),
        config
            .worker_jobs_root
            .as_ref()
            .map(|value| Value::String(value.display().to_string()))
            .unwrap_or(Value::Null),
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
        "session_secret".to_string(),
        Value::String(config.session_secret.clone()),
    );
    settings.insert(
        "bootstrap_completed".to_string(),
        Value::Bool(config.bootstrap_completed),
    );
    settings
}

fn is_non_runtime_dynamic_key(key: &str) -> bool {
    matches!(key, "signing_enabled" | "signing_key_id")
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
