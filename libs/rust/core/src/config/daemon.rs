use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    constants::{
        DAEMON_RUNTIME_ROOT, DAEMON_WORKER_RUNTIME_ROOT, DATABASE_URL_ENV_VAR, REDIS_URL_ENV_VAR,
        WORKER_JOBS_HOST_PATH_ENV_VAR,
    },
    error::SynforgeError,
    runtime::RuntimePaths,
};

use super::defaults::{
    default_bootstrap_completed, default_build_failure_backoff_base_seconds,
    default_build_failure_backoff_max_seconds, default_db_pool_size,
    default_git_mirror_max_unused_seconds, default_git_mirror_refresh_ttl_seconds,
    default_git_operation_timeout_seconds, default_listen_addr, default_max_concurrent_builds,
    default_mock_chroot_cache_ttl_seconds, default_poller_tick_seconds, default_public_base_url,
    default_queue_buffer_size, default_redis_key_prefix, default_signing_enabled,
    default_worker_image, default_worker_result_timeout_seconds,
    default_worker_socket_timeout_seconds,
};
use super::object_storage::{ObjectStorageConfig, load_object_storage_config_from_env};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonConfig {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    pub database_url: String,
    pub redis_url: String,
    #[serde(default = "default_redis_key_prefix")]
    pub redis_key_prefix: String,
    #[serde(default)]
    pub object_storage: ObjectStorageConfig,
    #[serde(default = "default_worker_image")]
    pub worker_image: String,
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
            object_storage: ObjectStorageConfig::default(),
            worker_image: default_worker_image(),
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
        let object_storage = &self.object_storage;
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
