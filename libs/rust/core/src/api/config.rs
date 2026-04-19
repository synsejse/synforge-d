use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct EffectiveConfigView {
    pub bootstrap_completed: bool,
    pub listen_addr: String,
    pub database_url: String,
    pub redis_url: String,
    pub redis_key_prefix: String,
    #[schema(value_type = String)]
    pub repo_dir: std::path::PathBuf,
    #[schema(value_type = String)]
    pub jobs_root: std::path::PathBuf,
    #[schema(value_type = String)]
    pub cache_root: std::path::PathBuf,
    #[schema(value_type = String)]
    pub work_root: std::path::PathBuf,
    #[schema(value_type = String)]
    pub signing_root: std::path::PathBuf,
    #[schema(value_type = String)]
    pub worker_jobs_root: std::path::PathBuf,
    #[schema(value_type = String)]
    pub worker_ccache_root: std::path::PathBuf,
    pub worker_image: String,
    pub signing_enabled: bool,
    pub signing_key_id: Option<String>,
    pub max_concurrent_builds: usize,
    pub db_pool_size: u32,
    pub queue_buffer_size: usize,
    pub poller_tick_seconds: u64,
    pub worker_result_timeout_seconds: u64,
    pub worker_socket_timeout_seconds: u64,
    pub git_operation_timeout_seconds: u64,
    pub public_base_url: String,
    pub mock_chroot_cache_ttl_seconds: u64,
    pub git_mirror_refresh_ttl_seconds: u64,
    pub git_mirror_max_unused_seconds: u64,
    pub build_failure_backoff_base_seconds: u64,
    pub build_failure_backoff_max_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct EffectiveConfigDto {
    pub config: EffectiveConfigView,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UpdateRuntimeSettingsRequest {
    #[serde(default)]
    pub settings: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConfigFieldType {
    String,
    Number,
    Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ConfigFieldDescriptor {
    pub key: String,
    pub label: String,
    pub description: String,
    pub section_key: String,
    pub section_label: String,
    #[serde(rename = "type")]
    pub field_type: ConfigFieldType,
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_value: Option<u64>,
    pub editable_in_setup: bool,
    pub editable_in_runtime: bool,
    pub default_value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ConfigSchemaResponse {
    pub fields: Vec<ConfigFieldDescriptor>,
}
