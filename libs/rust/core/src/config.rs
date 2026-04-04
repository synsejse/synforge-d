use std::path::PathBuf;

use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::{error::SynforgeError, runtime::RuntimePaths};

fn default_listen_addr() -> String {
    "0.0.0.0:8080".to_string()
}

fn default_runtime_root() -> PathBuf {
    PathBuf::from("/var/lib/synforge")
}

fn default_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://synforge:synforge_dev@localhost:3306/synforge".to_string())
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

fn default_session_secret() -> String {
    "synforge-dev-session-secret-change-me".to_string()
}

fn default_bootstrap_completed() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonConfig {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    #[serde(default = "default_runtime_root")]
    pub runtime_root: PathBuf,
    #[serde(default = "default_database_url")]
    pub database_url: String,
    #[serde(default = "default_worker_image")]
    pub worker_image: String,
    #[serde(default = "default_session_secret")]
    pub session_secret: String,
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
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_listen_addr(),
            runtime_root: default_runtime_root(),
            database_url: default_database_url(),
            worker_image: default_worker_image(),
            session_secret: default_session_secret(),
            bootstrap_completed: default_bootstrap_completed(),
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
    pub fn load() -> anyhow::Result<Self> {
        let path = Self::config_path();
        let mut config = if path.exists() {
            Self::load_from_file(&path)?
        } else {
            let config = Self {
                session_secret: generate_session_secret(),
                ..Self::default()
            };
            config.save_to_file(&path)?;
            config
        };
        if session_secret_needs_generation(&config.session_secret) {
            config.session_secret = generate_session_secret();
            config.save_to_file(&path)?;
        }
        config
            .validate()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        Ok(config)
    }

    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        let mut config: Self = serde_yaml::from_str(&raw)?;
        if config.runtime_root.as_os_str().is_empty() {
            config.runtime_root = default_runtime_root();
        }
        Ok(config)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.save_to_file(&Self::config_path())
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        self.validate()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = serde_yaml::to_string(self)?;
        write_config_atomically(path, &contents)
    }

    pub fn config_path() -> PathBuf {
        if let Some(value) = env_string("SYNFORGE_CONFIG_PATH") {
            return PathBuf::from(value);
        }
        default_runtime_root().join("config/config.yaml")
    }

    pub fn validate(&self) -> Result<(), SynforgeError> {
        if self.listen_addr.trim().is_empty() {
            return Err(SynforgeError::Config(
                "listen_addr must not be empty".to_string(),
            ));
        }
        if self.runtime_root.as_os_str().is_empty() {
            return Err(SynforgeError::Config(
                "runtime_root must not be empty".to_string(),
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
        Ok(())
    }

    pub fn runtime_paths(&self) -> RuntimePaths {
        RuntimePaths::new(self.runtime_root.clone())
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

fn session_secret_needs_generation(value: &str) -> bool {
    let value = value.trim();
    value.is_empty() || value == default_session_secret()
}

fn env_string(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn write_config_atomically(path: &std::path::Path, contents: &str) -> anyhow::Result<()> {
    let temp_path = path.with_extension("yaml.tmp");
    std::fs::write(&temp_path, contents)?;
    std::fs::rename(&temp_path, path)?;
    Ok(())
}
