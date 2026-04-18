use crate::api::{ConfigFieldDescriptor, ConfigFieldType};
use serde_json::Value;

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

pub fn editable_config_fields() -> Vec<ConfigFieldDescriptor> {
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
        config_number_field(
            ConfigSection {
                key: "scheduler",
                label: "Scheduler",
            },
            "build_failure_backoff_base_seconds",
            "Build failure backoff base seconds",
            "Base delay for failure backoff; each consecutive failure doubles this delay.",
            300,
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
            "build_failure_backoff_max_seconds",
            "Build failure backoff max seconds",
            "Maximum delay cap for exponential failure backoff.",
            21_600,
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
