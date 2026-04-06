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
use crate::db::UserStore;

impl SynforgeService {
    pub async fn config_schema(&self) -> ConfigSchemaResponse {
        ConfigSchemaResponse {
            fields: editable_config_fields(),
        }
    }

    pub async fn effective_config(&self) -> EffectiveConfigDto {
        let current = DaemonConfig::load().unwrap_or_else(|_| self.config.clone());
        let paths = current.runtime_paths();
        EffectiveConfigDto {
            config: EffectiveConfigView {
                config_path: DaemonConfig::config_path(),
                bootstrap_completed: current.bootstrap_completed,
                listen_addr: current.listen_addr.clone(),
                runtime_root: current.runtime_root.clone(),
                database_url: current.database_url.clone(),
                packages_dir: paths.packages_dir().to_path_buf(),
                repo_dir: paths.repo_dir().to_path_buf(),
                jobs_root: paths.jobs_root().to_path_buf(),
                worker_image: current.worker_image.clone(),
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
        }
    }

    pub async fn update_runtime_settings(
        &self,
        request: UpdateRuntimeSettingsRequest,
    ) -> anyhow::Result<EffectiveConfigDto> {
        let mut config = DaemonConfig::load()?;
        apply_config_settings(&mut config, &request.settings, false)?;
        config.save()?;
        Ok(self.effective_config().await)
    }

    pub async fn initialize_setup(
        &self,
        request: SetupInitializeRequest,
    ) -> anyhow::Result<EffectiveConfigDto> {
        let current = DaemonConfig::load()?;
        if current.bootstrap_completed {
            anyhow::bail!("setup has already been completed");
        }
        super::validate_user_handle(&request.admin.handle)?;
        super::validate_display_name(&request.admin.display_name)?;
        super::validate_password(&request.admin.password)?;
        let mut config = current;
        apply_config_settings(&mut config, &request.settings, true)?;
        config.bootstrap_completed = false;
        config
            .validate()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        if self.store.user_count().await? > 0 {
            anyhow::bail!("initial admin cannot be created because users already exist");
        }
        config.save()?;
        self.bootstrap_admin(
            &request.admin.handle,
            &request.admin.display_name,
            &request.admin.password,
        )
        .await?;
        config.bootstrap_completed = true;
        config.save()?;
        Ok(self.effective_config().await)
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

fn apply_config_settings(
    config: &mut DaemonConfig,
    settings: &BTreeMap<String, Value>,
    allow_setup_only: bool,
) -> anyhow::Result<()> {
    for key in settings.keys() {
        let Some(field) = editable_config_fields()
            .into_iter()
            .find(|field| field.key == *key)
        else {
            anyhow::bail!("unknown config setting: {key}");
        };
        if allow_setup_only {
            if !field.editable_in_setup {
                anyhow::bail!("config setting is not editable during setup: {key}");
            }
        } else if !field.editable_in_runtime {
            anyhow::bail!("config setting is not editable at runtime: {key}");
        }
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

    Ok(())
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
