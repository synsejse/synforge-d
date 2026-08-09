use async_trait::async_trait;
use synforge_core::model::{UserPermission, UserSummary};
use uuid::Uuid;

use super::super::UserAuthRecord;

#[async_trait]
pub trait UserStore: Send + Sync {
    async fn user_count(&self) -> anyhow::Result<u64>;
    async fn list_users(&self) -> anyhow::Result<Vec<UserSummary>>;
    async fn list_users_paginated(
        &self,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<Vec<UserSummary>>;
    async fn get_user(&self, user_id: Uuid) -> anyhow::Result<Option<UserSummary>>;
    async fn get_bootstrap_admin(&self) -> anyhow::Result<Option<UserSummary>>;
    async fn get_user_by_handle(&self, handle: &str) -> anyhow::Result<Option<UserSummary>>;
    async fn get_user_auth_by_handle(&self, handle: &str)
    -> anyhow::Result<Option<UserAuthRecord>>;

    async fn create_user(
        &self,
        handle: &str,
        display_name: &str,
        password_hash: &str,
        active: bool,
        is_bootstrap_admin: bool,
        permissions: &[UserPermission],
    ) -> anyhow::Result<UserSummary>;

    async fn update_user(
        &self,
        user_id: Uuid,
        handle: &str,
        display_name: &str,
        active: bool,
        permissions: &[UserPermission],
    ) -> anyhow::Result<Option<UserSummary>>;

    async fn update_user_password(
        &self,
        user_id: Uuid,
        password_hash: &str,
    ) -> anyhow::Result<bool>;

    async fn delete_user(&self, user_id: Uuid) -> anyhow::Result<Option<UserSummary>>;
    async fn increment_user_download_bytes(&self, user_id: Uuid, bytes: u64) -> anyhow::Result<()>;
}
