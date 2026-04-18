use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::model::{UserAccount, UserPermission, UserRepoMetrics};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UserResponse {
    pub user: UserAccount,
    pub metrics: UserRepoMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CreateUserRequest {
    pub handle: String,
    pub display_name: String,
    pub password: String,
    pub permissions: Vec<UserPermission>,
    #[serde(default = "default_user_active")]
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UpdateUserRequest {
    pub handle: String,
    pub display_name: String,
    pub permissions: Vec<UserPermission>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ChangePasswordRequest {
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SessionLoginRequest {
    pub handle: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SessionResponse {
    pub user: UserAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SetupStatusResponse {
    pub initialized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SetupAdminRequest {
    pub handle: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SetupInitializeRequest {
    #[serde(default)]
    pub settings: BTreeMap<String, Value>,
    #[serde(default = "default_setup_enable_signing")]
    pub enable_signing: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signing_armored_private_key: Option<String>,
    pub admin: SetupAdminRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UserMetricsResponse {
    pub metrics: UserRepoMetrics,
}

fn default_user_active() -> bool {
    true
}

fn default_setup_enable_signing() -> bool {
    true
}
