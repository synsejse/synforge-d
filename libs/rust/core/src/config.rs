use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{RuntimePaths, SynforgeError};

fn default_listen_addr() -> String {
    "0.0.0.0:8080".to_string()
}

fn default_runtime_root() -> PathBuf {
    PathBuf::from("/var/lib/synforge")
}

fn default_max_concurrent_builds() -> usize {
    2
}

fn default_public_base_url() -> String {
    "http://localhost:8080".to_string()
}

fn default_worker_listen_addr() -> String {
    "0.0.0.0:8090".to_string()
}

fn default_worker_connect_addr() -> String {
    "host.docker.internal:8090".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonConfig {
    pub listen_addr: String,
    pub bearer_token: String,
    pub runtime_root: PathBuf,
    pub worker_image: String,
    #[serde(default = "default_max_concurrent_builds")]
    pub max_concurrent_builds: usize,
    pub public_base_url: String,
    pub worker_listen_addr: String,
    pub worker_connect_addr: String,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_listen_addr(),
            bearer_token: String::new(),
            runtime_root: default_runtime_root(),
            worker_image: default_worker_image(),
            max_concurrent_builds: default_max_concurrent_builds(),
            public_base_url: default_public_base_url(),
            worker_listen_addr: default_worker_listen_addr(),
            worker_connect_addr: default_worker_connect_addr(),
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
            public_base_url: env_string("SYNFORGE_PUBLIC_BASE_URL").unwrap_or_else(default_public_base_url),
            worker_listen_addr: env_string("SYNFORGE_WORKER_LISTEN_ADDR")
                .unwrap_or_else(default_worker_listen_addr),
            worker_connect_addr: env_string("SYNFORGE_WORKER_CONNECT_ADDR")
                .unwrap_or_else(default_worker_connect_addr),
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
        if self.worker_listen_addr.trim().is_empty() {
            return Err(SynforgeError::Config(
                "worker_listen_addr must not be empty".to_string(),
            ));
        }
        if self.worker_connect_addr.trim().is_empty() {
            return Err(SynforgeError::Config(
                "worker_connect_addr must not be empty".to_string(),
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
