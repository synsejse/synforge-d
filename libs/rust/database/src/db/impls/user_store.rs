use async_trait::async_trait;
use synforge_core::model::{UserPermission, UserSummary};
use uuid::Uuid;

use crate::db::{DieselStore, UserAuthRecord, UserStore, user};

#[async_trait]
impl UserStore for DieselStore {
    async fn user_count(&self) -> anyhow::Result<u64> {
        user::user_count(self).await
    }

    async fn list_users(&self) -> anyhow::Result<Vec<UserSummary>> {
        user::list_users(self).await
    }

    async fn get_user(&self, user_id: Uuid) -> anyhow::Result<Option<UserSummary>> {
        user::get_user(self, user_id).await
    }

    async fn get_user_by_handle(&self, handle: &str) -> anyhow::Result<Option<UserSummary>> {
        user::get_user_by_handle(self, handle).await
    }

    async fn get_user_auth_by_handle(
        &self,
        handle: &str,
    ) -> anyhow::Result<Option<UserAuthRecord>> {
        user::get_user_auth_by_handle(self, handle).await
    }

    async fn create_user(
        &self,
        handle: &str,
        display_name: &str,
        password_hash: &str,
        active: bool,
        permissions: &[UserPermission],
    ) -> anyhow::Result<UserSummary> {
        user::create_user(
            self,
            handle,
            display_name,
            password_hash,
            active,
            permissions,
        )
        .await
    }

    async fn update_user(
        &self,
        user_id: Uuid,
        handle: &str,
        display_name: &str,
        active: bool,
        permissions: &[UserPermission],
    ) -> anyhow::Result<Option<UserSummary>> {
        user::update_user(self, user_id, handle, display_name, active, permissions).await
    }

    async fn update_user_password(
        &self,
        user_id: Uuid,
        password_hash: &str,
    ) -> anyhow::Result<bool> {
        user::update_user_password(self, user_id, password_hash).await
    }

    async fn delete_user(&self, user_id: Uuid) -> anyhow::Result<Option<UserSummary>> {
        user::delete_user(self, user_id).await
    }

    async fn increment_user_download_bytes(&self, user_id: Uuid, bytes: u64) -> anyhow::Result<()> {
        user::increment_user_download_bytes(self, user_id, bytes).await
    }
}
