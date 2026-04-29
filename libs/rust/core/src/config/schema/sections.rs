use super::builders::{
    ConfigEditability, ConfigSection, config_bool_field, config_number_field,
    config_optional_string_field, config_string_field,
};
use crate::api::ConfigFieldDescriptor;

const SETUP_AND_RUNTIME: ConfigEditability = ConfigEditability {
    in_setup: true,
    in_runtime: true,
};
const SETUP_ONLY: ConfigEditability = ConfigEditability {
    in_setup: true,
    in_runtime: false,
};
const READ_ONLY: ConfigEditability = ConfigEditability {
    in_setup: false,
    in_runtime: false,
};

pub(super) fn server_fields() -> Vec<ConfigFieldDescriptor> {
    let server = ConfigSection {
        key: "server",
        label: "Server",
    };
    vec![
        config_string_field(
            server,
            "listen_addr",
            "Listen address",
            "Daemon HTTP listen address.",
            "0.0.0.0:8080",
            SETUP_ONLY,
        ),
        config_string_field(
            server,
            "public_base_url",
            "Public base URL",
            "Base URL used in generated links and repo setup.",
            "http://localhost:8080",
            SETUP_AND_RUNTIME,
        ),
    ]
}

pub(super) fn worker_fields() -> Vec<ConfigFieldDescriptor> {
    let worker = ConfigSection {
        key: "worker",
        label: "Worker",
    };
    vec![
        config_string_field(
            worker,
            "worker_image",
            "Worker image",
            "Docker image used for spawned worker containers.",
            "synforge-worker-fedora:latest",
            SETUP_AND_RUNTIME,
        ),
        config_number_field(
            worker,
            "worker_result_timeout_seconds",
            "Worker result timeout seconds",
            "Timeout while waiting for worker completion after request dispatch.",
            10,
            SETUP_AND_RUNTIME,
        ),
        config_number_field(
            worker,
            "worker_socket_timeout_seconds",
            "Worker socket timeout seconds",
            "Socket timeout used for worker protocol I/O.",
            30,
            SETUP_AND_RUNTIME,
        ),
    ]
}

pub(super) fn signing_fields() -> Vec<ConfigFieldDescriptor> {
    let signing = ConfigSection {
        key: "signing",
        label: "Signing",
    };
    vec![
        config_bool_field(
            signing,
            "signing_enabled",
            "Repository GPG signing enabled",
            "Enable package signing for repository artifacts.",
            false,
            READ_ONLY,
        ),
        config_optional_string_field(
            signing,
            "signing_key_id",
            "Repository GPG key ID",
            "Configured signing key identifier. Managed by the dedicated signing APIs.",
            READ_ONLY,
        ),
    ]
}

pub(super) fn build_fields() -> Vec<ConfigFieldDescriptor> {
    let build = ConfigSection {
        key: "build",
        label: "Build",
    };
    vec![config_number_field(
        build,
        "max_concurrent_builds",
        "Max concurrent builds",
        "Maximum number of active builds at once.",
        2,
        SETUP_AND_RUNTIME,
    )]
}

pub(super) fn database_fields() -> Vec<ConfigFieldDescriptor> {
    let database = ConfigSection {
        key: "database",
        label: "Database",
    };
    vec![config_number_field(
        database,
        "db_pool_size",
        "DB pool size",
        "Number of database connection pool slots.",
        5,
        SETUP_ONLY,
    )]
}

pub(super) fn scheduler_fields() -> Vec<ConfigFieldDescriptor> {
    let scheduler = ConfigSection {
        key: "scheduler",
        label: "Scheduler",
    };
    vec![
        config_number_field(
            scheduler,
            "queue_buffer_size",
            "Queue buffer size",
            "In-memory queued build channel capacity.",
            128,
            SETUP_AND_RUNTIME,
        ),
        config_number_field(
            scheduler,
            "poller_tick_seconds",
            "Poller tick seconds",
            "How often package polling wakes up.",
            30,
            SETUP_AND_RUNTIME,
        ),
        config_number_field(
            scheduler,
            "build_failure_backoff_base_seconds",
            "Build failure backoff base seconds",
            "Base delay for failure backoff; each consecutive failure doubles this delay.",
            300,
            SETUP_AND_RUNTIME,
        ),
        config_number_field(
            scheduler,
            "build_failure_backoff_max_seconds",
            "Build failure backoff max seconds",
            "Maximum delay cap for exponential failure backoff.",
            21_600,
            SETUP_AND_RUNTIME,
        ),
    ]
}

pub(super) fn git_fields() -> Vec<ConfigFieldDescriptor> {
    let git = ConfigSection {
        key: "git",
        label: "Git",
    };
    vec![config_number_field(
        git,
        "git_operation_timeout_seconds",
        "Git operation timeout seconds",
        "Timeout applied to git inspection and sync commands.",
        600,
        SETUP_AND_RUNTIME,
    )]
}

pub(super) fn cache_fields() -> Vec<ConfigFieldDescriptor> {
    let cache = ConfigSection {
        key: "cache",
        label: "Cache",
    };
    vec![
        config_number_field(
            cache,
            "mock_chroot_cache_ttl_seconds",
            "Mock chroot cache TTL seconds",
            "How long to cache discovered mock chroots before refreshing.",
            300,
            SETUP_AND_RUNTIME,
        ),
        config_number_field(
            cache,
            "git_mirror_refresh_ttl_seconds",
            "Git mirror refresh TTL seconds",
            "Maximum age before a cached git mirror is refreshed from origin.",
            300,
            SETUP_AND_RUNTIME,
        ),
        config_number_field(
            cache,
            "git_mirror_max_unused_seconds",
            "Git mirror max unused seconds",
            "Remove cached git mirrors that have not been used within this window.",
            604_800,
            SETUP_AND_RUNTIME,
        ),
    ]
}
