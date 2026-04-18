use serde::{Deserialize, Serialize};

use crate::constants::{
    OBJECT_STORAGE_ACCESS_KEY_ID_ENV_VAR, OBJECT_STORAGE_BUCKET_ENV_VAR,
    OBJECT_STORAGE_ENDPOINT_ENV_VAR, OBJECT_STORAGE_PATH_STYLE_ENV_VAR,
    OBJECT_STORAGE_REGION_ENV_VAR, OBJECT_STORAGE_SECRET_ACCESS_KEY_ENV_VAR,
};

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

impl Default for ObjectStorageConfig {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            region: "us-east-1".to_string(),
            bucket: String::new(),
            access_key_id: String::new(),
            secret_access_key: String::new(),
            path_style: default_object_storage_path_style(),
        }
    }
}

pub(super) fn load_object_storage_config_from_env() -> anyhow::Result<ObjectStorageConfig> {
    let endpoint = std::env::var(OBJECT_STORAGE_ENDPOINT_ENV_VAR)
        .map(|value| value.trim().to_string())
        .map_err(|_| {
            anyhow::anyhow!(
                "{} environment variable is required but not set",
                OBJECT_STORAGE_ENDPOINT_ENV_VAR
            )
        })?;

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

    Ok(ObjectStorageConfig {
        endpoint,
        region,
        bucket,
        access_key_id,
        secret_access_key,
        path_style,
    })
}
