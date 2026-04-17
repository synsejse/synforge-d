use std::path::PathBuf;

use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{
        DAEMON_RUNTIME_ROOT, DAEMON_WORKER_RUNTIME_ROOT, DATABASE_URL_ENV_VAR,
        DEFAULT_DAEMON_LISTEN_ADDR, DEFAULT_DAEMON_PUBLIC_BASE_URL,
        OBJECT_STORAGE_ACCESS_KEY_ID_ENV_VAR, OBJECT_STORAGE_BUCKET_ENV_VAR,
        OBJECT_STORAGE_ENDPOINT_ENV_VAR, OBJECT_STORAGE_PATH_STYLE_ENV_VAR,
        OBJECT_STORAGE_REGION_ENV_VAR, OBJECT_STORAGE_SECRET_ACCESS_KEY_ENV_VAR, REDIS_URL_ENV_VAR,
        WORKER_JOBS_HOST_PATH_ENV_VAR,
    },
    error::SynforgeError,
    runtime::RuntimePaths,
};

fn default_listen_addr() -> String {
    DEFAULT_DAEMON_LISTEN_ADDR.to_string()
}

fn default_max_concurrent_builds() -> usize {
    2
}

fn default_db_pool_size() -> u32 {
    5
}

fn default_queue_buffer_size() -> usize {
    128
}

fn default_poller_tick_seconds() -> u64 {
    30
}

fn default_worker_result_timeout_seconds() -> u64 {
    10
}

fn default_worker_socket_timeout_seconds() -> u64 {
    30
}

fn default_git_operation_timeout_seconds() -> u64 {
    600
}

fn default_public_base_url() -> String {
    DEFAULT_DAEMON_PUBLIC_BASE_URL.to_string()
}

fn default_redis_key_prefix() -> String {
    "synforge".to_string()
}

fn default_mock_chroot_cache_ttl_seconds() -> u64 {
    300
}

fn default_git_mirror_refresh_ttl_seconds() -> u64 {
    300
}

fn default_git_mirror_max_unused_seconds() -> u64 {
    604_800
}

fn default_build_failure_backoff_base_seconds() -> u64 {
    300
}

fn default_build_failure_backoff_max_seconds() -> u64 {
    21_600
}

fn default_session_secret() -> String {
    generate_session_secret()
}

fn default_signing_enabled() -> bool {
    false
}

fn default_bootstrap_completed() -> bool {
    false
}

fn default_object_storage_path_style() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectStorageConfig {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    #[serde(default = "default_object_storage_path_style")]
    pub path_style: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonConfig {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    pub database_url: String,
    pub redis_url: String,
    #[serde(default = "default_redis_key_prefix")]
    pub redis_key_prefix: String,
    #[serde(default)]
    pub object_storage: Option<ObjectStorageConfig>,
    #[serde(default = "default_worker_image")]
    pub worker_image: String,
    #[serde(default = "default_session_secret")]
    pub session_secret: String,
    #[serde(default = "default_signing_enabled")]
    pub signing_enabled: bool,
    #[serde(default)]
    pub signing_key_id: Option<String>,
    #[serde(default = "default_bootstrap_completed")]
    pub bootstrap_completed: bool,
    #[serde(default = "default_max_concurrent_builds")]
    pub max_concurrent_builds: usize,
    #[serde(default = "default_db_pool_size")]
    pub db_pool_size: u32,
    #[serde(default = "default_queue_buffer_size")]
    pub queue_buffer_size: usize,
    #[serde(default = "default_poller_tick_seconds")]
    pub poller_tick_seconds: u64,
    #[serde(default = "default_worker_result_timeout_seconds")]
    pub worker_result_timeout_seconds: u64,
    #[serde(default = "default_worker_socket_timeout_seconds")]
    pub worker_socket_timeout_seconds: u64,
    #[serde(default = "default_git_operation_timeout_seconds")]
    pub git_operation_timeout_seconds: u64,
    #[serde(default = "default_public_base_url")]
    pub public_base_url: String,
    #[serde(default = "default_mock_chroot_cache_ttl_seconds")]
    pub mock_chroot_cache_ttl_seconds: u64,
    #[serde(default = "default_git_mirror_refresh_ttl_seconds")]
    pub git_mirror_refresh_ttl_seconds: u64,
    #[serde(default = "default_git_mirror_max_unused_seconds")]
    pub git_mirror_max_unused_seconds: u64,
    #[serde(default = "default_build_failure_backoff_base_seconds")]
    pub build_failure_backoff_base_seconds: u64,
    #[serde(default = "default_build_failure_backoff_max_seconds")]
    pub build_failure_backoff_max_seconds: u64,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_listen_addr(),
            database_url: String::new(),
            redis_url: String::new(),
            redis_key_prefix: default_redis_key_prefix(),
            object_storage: None,
            worker_image: default_worker_image(),
            session_secret: default_session_secret(),
            signing_enabled: default_signing_enabled(),
            signing_key_id: None,
            bootstrap_completed: default_bootstrap_completed(),
            max_concurrent_builds: default_max_concurrent_builds(),
            db_pool_size: default_db_pool_size(),
            queue_buffer_size: default_queue_buffer_size(),
            poller_tick_seconds: default_poller_tick_seconds(),
            worker_result_timeout_seconds: default_worker_result_timeout_seconds(),
            worker_socket_timeout_seconds: default_worker_socket_timeout_seconds(),
            git_operation_timeout_seconds: default_git_operation_timeout_seconds(),
            public_base_url: default_public_base_url(),
            mock_chroot_cache_ttl_seconds: default_mock_chroot_cache_ttl_seconds(),
            git_mirror_refresh_ttl_seconds: default_git_mirror_refresh_ttl_seconds(),
            git_mirror_max_unused_seconds: default_git_mirror_max_unused_seconds(),
            build_failure_backoff_base_seconds: default_build_failure_backoff_base_seconds(),
            build_failure_backoff_max_seconds: default_build_failure_backoff_max_seconds(),
        }
    }
}

impl DaemonConfig {
    pub fn load() -> anyhow::Result<Self> {
        let database_url = std::env::var(DATABASE_URL_ENV_VAR).map_err(|_| {
            anyhow::anyhow!(
                "{} environment variable is required but not set",
                DATABASE_URL_ENV_VAR
            )
        })?;
        let redis_url = std::env::var(REDIS_URL_ENV_VAR).map_err(|_| {
            anyhow::anyhow!(
                "{} environment variable is required but not set",
                REDIS_URL_ENV_VAR
            )
        })?;
        let object_storage = load_object_storage_config_from_env()?;
        let config = Self {
            database_url,
            redis_url,
            object_storage,
            ..Self::default()
        };
        config
            .validate()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), SynforgeError> {
        if self.listen_addr.trim().is_empty() {
            return Err(SynforgeError::Config(
                "listen_addr must not be empty".to_string(),
            ));
        }
        if !matches!(
            self.database_url.split(':').next(),
            Some("postgres") | Some("postgresql")
        ) {
            return Err(SynforgeError::Config(
                "database_url must use a PostgreSQL URL".to_string(),
            ));
        }
        if self.redis_url.trim().is_empty() {
            return Err(SynforgeError::Config(
                "redis_url must not be empty".to_string(),
            ));
        }
        if self.redis_key_prefix.trim().is_empty() {
            return Err(SynforgeError::Config(
                "redis_key_prefix must not be empty".to_string(),
            ));
        }
        if self.max_concurrent_builds == 0 {
            return Err(SynforgeError::Config(
                "max_concurrent_builds must be greater than zero".to_string(),
            ));
        }
        if self.db_pool_size == 0 {
            return Err(SynforgeError::Config(
                "db_pool_size must be greater than zero".to_string(),
            ));
        }
        if self.queue_buffer_size == 0 {
            return Err(SynforgeError::Config(
                "queue_buffer_size must be greater than zero".to_string(),
            ));
        }
        if self.poller_tick_seconds == 0 {
            return Err(SynforgeError::Config(
                "poller_tick_seconds must be greater than zero".to_string(),
            ));
        }
        if self.worker_result_timeout_seconds == 0 {
            return Err(SynforgeError::Config(
                "worker_result_timeout_seconds must be greater than zero".to_string(),
            ));
        }
        if self.worker_socket_timeout_seconds == 0 {
            return Err(SynforgeError::Config(
                "worker_socket_timeout_seconds must be greater than zero".to_string(),
            ));
        }
        if self.git_operation_timeout_seconds == 0 {
            return Err(SynforgeError::Config(
                "git_operation_timeout_seconds must be greater than zero".to_string(),
            ));
        }
        if self.worker_image.trim().is_empty() {
            return Err(SynforgeError::Config(
                "worker_image must not be empty".to_string(),
            ));
        }
        if self.session_secret.trim().is_empty() {
            return Err(SynforgeError::Config(
                "session_secret must not be empty".to_string(),
            ));
        }
        if let Some(key_id) = self.signing_key_id.as_ref()
            && key_id.trim().is_empty()
        {
            return Err(SynforgeError::Config(
                "signing_key_id must not be empty when provided".to_string(),
            ));
        }
        if self.mock_chroot_cache_ttl_seconds == 0 {
            return Err(SynforgeError::Config(
                "mock_chroot_cache_ttl_seconds must be greater than zero".to_string(),
            ));
        }
        if self.git_mirror_refresh_ttl_seconds == 0 {
            return Err(SynforgeError::Config(
                "git_mirror_refresh_ttl_seconds must be greater than zero".to_string(),
            ));
        }
        if self.git_mirror_max_unused_seconds == 0 {
            return Err(SynforgeError::Config(
                "git_mirror_max_unused_seconds must be greater than zero".to_string(),
            ));
        }
        if self.build_failure_backoff_base_seconds == 0 {
            return Err(SynforgeError::Config(
                "build_failure_backoff_base_seconds must be greater than zero".to_string(),
            ));
        }
        if self.build_failure_backoff_max_seconds == 0 {
            return Err(SynforgeError::Config(
                "build_failure_backoff_max_seconds must be greater than zero".to_string(),
            ));
        }
        if self.build_failure_backoff_max_seconds < self.build_failure_backoff_base_seconds {
            return Err(SynforgeError::Config(
                "build_failure_backoff_max_seconds must be greater than or equal to build_failure_backoff_base_seconds".to_string(),
            ));
        }
        if let Some(object_storage) = self.object_storage.as_ref() {
            if object_storage.endpoint.trim().is_empty() {
                return Err(SynforgeError::Config(
                    "object_storage.endpoint must not be empty".to_string(),
                ));
            }
            if object_storage.region.trim().is_empty() {
                return Err(SynforgeError::Config(
                    "object_storage.region must not be empty".to_string(),
                ));
            }
            if object_storage.bucket.trim().is_empty() {
                return Err(SynforgeError::Config(
                    "object_storage.bucket must not be empty".to_string(),
                ));
            }
            if object_storage.access_key_id.trim().is_empty() {
                return Err(SynforgeError::Config(
                    "object_storage.access_key_id must not be empty".to_string(),
                ));
            }
            if object_storage.secret_access_key.trim().is_empty() {
                return Err(SynforgeError::Config(
                    "object_storage.secret_access_key must not be empty".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn runtime_paths(&self) -> RuntimePaths {
        RuntimePaths::new(PathBuf::from(DAEMON_RUNTIME_ROOT))
    }

    pub fn worker_runtime_root(&self) -> PathBuf {
        PathBuf::from(DAEMON_WORKER_RUNTIME_ROOT)
    }

    pub fn worker_jobs_root(&self) -> PathBuf {
        self.worker_runtime_root().join("jobs")
    }

    pub fn worker_ccache_root(&self) -> PathBuf {
        self.worker_runtime_root().join("cache").join("ccache")
    }

    /// Returns the host-side path for the worker runtime root, used for Docker bind mounts.
    /// This is required when daemon runs in a container but spawns workers via Docker API.
    pub fn worker_runtime_host_path(&self) -> Option<PathBuf> {
        std::env::var(WORKER_JOBS_HOST_PATH_ENV_VAR)
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
    }

    pub fn worker_jobs_host_path(&self) -> Option<PathBuf> {
        self.worker_runtime_host_path()
            .map(|root| root.join("jobs"))
    }

    pub fn worker_ccache_host_path(&self) -> Option<PathBuf> {
        self.worker_runtime_host_path()
            .map(|root| root.join("cache").join("ccache"))
    }
}

fn default_worker_image() -> String {
    "synforge-worker-fedora:latest".to_string()
}

fn generate_session_secret() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn load_object_storage_config_from_env() -> anyhow::Result<Option<ObjectStorageConfig>> {
    let endpoint = std::env::var(OBJECT_STORAGE_ENDPOINT_ENV_VAR)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let Some(endpoint) = endpoint else {
        return Ok(None);
    };

    let region = std::env::var(OBJECT_STORAGE_REGION_ENV_VAR)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "us-east-1".to_string());
    let bucket = std::env::var(OBJECT_STORAGE_BUCKET_ENV_VAR)
        .map(|value| value.trim().to_string())
        .map_err(|_| {
            anyhow::anyhow!(
                "{} environment variable is required when {} is set",
                OBJECT_STORAGE_BUCKET_ENV_VAR,
                OBJECT_STORAGE_ENDPOINT_ENV_VAR
            )
        })?;
    let access_key_id = std::env::var(OBJECT_STORAGE_ACCESS_KEY_ID_ENV_VAR)
        .map(|value| value.trim().to_string())
        .map_err(|_| {
            anyhow::anyhow!(
                "{} environment variable is required when {} is set",
                OBJECT_STORAGE_ACCESS_KEY_ID_ENV_VAR,
                OBJECT_STORAGE_ENDPOINT_ENV_VAR
            )
        })?;
    let secret_access_key = std::env::var(OBJECT_STORAGE_SECRET_ACCESS_KEY_ENV_VAR)
        .map(|value| value.trim().to_string())
        .map_err(|_| {
            anyhow::anyhow!(
                "{} environment variable is required when {} is set",
                OBJECT_STORAGE_SECRET_ACCESS_KEY_ENV_VAR,
                OBJECT_STORAGE_ENDPOINT_ENV_VAR
            )
        })?;
    let path_style = std::env::var(OBJECT_STORAGE_PATH_STYLE_ENV_VAR)
        .ok()
        .map(|value| matches!(value.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or_else(default_object_storage_path_style);

    Ok(Some(ObjectStorageConfig {
        endpoint,
        region,
        bucket,
        access_key_id,
        secret_access_key,
        path_style,
    }))
}
