use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{error::SynforgeError, runtime::RuntimePaths};

fn default_listen_addr() -> String {
    "0.0.0.0:8080".to_string()
}

fn default_runtime_root() -> PathBuf {
    PathBuf::from("/var/lib/synforge")
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
    "http://localhost:8080".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonConfig {
    pub listen_addr: String,
    pub bearer_token: String,
    pub runtime_root: PathBuf,
    pub worker_image: String,
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
    pub public_base_url: String,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_listen_addr(),
            bearer_token: String::new(),
            runtime_root: default_runtime_root(),
            worker_image: default_worker_image(),
            max_concurrent_builds: default_max_concurrent_builds(),
            db_pool_size: default_db_pool_size(),
            queue_buffer_size: default_queue_buffer_size(),
            poller_tick_seconds: default_poller_tick_seconds(),
            worker_result_timeout_seconds: default_worker_result_timeout_seconds(),
            worker_socket_timeout_seconds: default_worker_socket_timeout_seconds(),
            git_operation_timeout_seconds: default_git_operation_timeout_seconds(),
            public_base_url: default_public_base_url(),
        }
    }
}

impl DaemonConfig {
    pub fn load_from_env() -> anyhow::Result<Self> {
        let config = Self {
            listen_addr: env_string("SYNFORGE_LISTEN_ADDR").unwrap_or_else(default_listen_addr),
            bearer_token: env_string("SYNFORGE_BEARER_TOKEN").unwrap_or_default(),
            runtime_root: env_string("SYNFORGE_RUNTIME_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(default_runtime_root),
            worker_image: env_string("SYNFORGE_WORKER_IMAGE").unwrap_or_else(default_worker_image),
            max_concurrent_builds: env_usize("SYNFORGE_MAX_CONCURRENT_BUILDS")
                .unwrap_or_else(default_max_concurrent_builds),
            db_pool_size: env_u32("SYNFORGE_DB_POOL_SIZE").unwrap_or_else(default_db_pool_size),
            queue_buffer_size: env_usize("SYNFORGE_QUEUE_BUFFER_SIZE")
                .unwrap_or_else(default_queue_buffer_size),
            poller_tick_seconds: env_u64("SYNFORGE_POLLER_TICK_SECONDS")
                .unwrap_or_else(default_poller_tick_seconds),
            worker_result_timeout_seconds: env_u64("SYNFORGE_WORKER_RESULT_TIMEOUT_SECONDS")
                .unwrap_or_else(default_worker_result_timeout_seconds),
            worker_socket_timeout_seconds: env_u64("SYNFORGE_WORKER_SOCKET_TIMEOUT_SECONDS")
                .unwrap_or_else(default_worker_socket_timeout_seconds),
            git_operation_timeout_seconds: env_u64("SYNFORGE_GIT_OPERATION_TIMEOUT_SECONDS")
                .unwrap_or_else(default_git_operation_timeout_seconds),
            public_base_url: env_string("SYNFORGE_PUBLIC_BASE_URL").unwrap_or_else(default_public_base_url),
        };
        config
            .validate()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), SynforgeError> {
        if self.listen_addr.trim().is_empty() {
            return Err(SynforgeError::Config("listen_addr must not be empty".to_string()));
        }
        if self.runtime_root.as_os_str().is_empty() {
            return Err(SynforgeError::Config("runtime_root must not be empty".to_string()));
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
        Ok(())
    }

    pub fn runtime_paths(&self) -> RuntimePaths {
        RuntimePaths::new(
            self.runtime_root.join("metadata/database/state.db"),
            self.runtime_root.join("metadata/packages"),
            self.runtime_root.join("metadata/repo/fedora"),
            self.runtime_root.join("jobs"),
        )
    }
}

fn default_worker_image() -> String {
    "synforge-worker-fedora:latest".to_string()
}

fn env_string(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn env_usize(name: &str) -> Option<usize> {
    env_string(name).and_then(|value| value.parse::<usize>().ok())
}

fn env_u32(name: &str) -> Option<u32> {
    env_string(name).and_then(|value| value.parse::<u32>().ok())
}

fn env_u64(name: &str) -> Option<u64> {
    env_string(name).and_then(|value| value.parse::<u64>().ok())
}
