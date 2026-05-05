use crate::{DieselStore, UserAuthRecord, UserStore};
use synforge_core::model::{UserPermission, UserSummary};
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresUserStore {
    store: DieselStore,
}

impl PostgresUserStore {
    pub fn new(store: DieselStore) -> Self {
        Self { store }
    }

    pub async fn user_count(&self) -> anyhow::Result<u64> {
        self.store.user_count().await
    }

    pub async fn list_users(&self) -> anyhow::Result<Vec<UserSummary>> {
        self.store.list_users().await
    }

    pub async fn get_user(&self, user_id: Uuid) -> anyhow::Result<Option<UserSummary>> {
        self.store.get_user(user_id).await
    }

    pub async fn get_user_by_handle(&self, handle: &str) -> anyhow::Result<Option<UserSummary>> {
        self.store.get_user_by_handle(handle).await
    }

    pub async fn get_user_auth_by_handle(
        &self,
        handle: &str,
    ) -> anyhow::Result<Option<UserAuthRecord>> {
        self.store.get_user_auth_by_handle(handle).await
    }

    pub async fn create_user(
        &self,
        handle: &str,
        display_name: &str,
        password_hash: &str,
        active: bool,
        permissions: &[UserPermission],
    ) -> anyhow::Result<UserSummary> {
        self.store
            .create_user(handle, display_name, password_hash, active, permissions)
            .await
    }

    pub async fn update_user(
        &self,
        user_id: Uuid,
        handle: &str,
        display_name: &str,
        active: bool,
        permissions: &[UserPermission],
    ) -> anyhow::Result<Option<UserSummary>> {
        self.store
            .update_user(user_id, handle, display_name, active, permissions)
            .await
    }

    pub async fn update_user_password(
        &self,
        user_id: Uuid,
        password_hash: &str,
    ) -> anyhow::Result<bool> {
        self.store
            .update_user_password(user_id, password_hash)
            .await
    }

    pub async fn delete_user(&self, user_id: Uuid) -> anyhow::Result<Option<UserSummary>> {
        self.store.delete_user(user_id).await
    }

    pub async fn increment_user_download_bytes(
        &self,
        user_id: Uuid,
        bytes: u64,
    ) -> anyhow::Result<()> {
        self.store
            .increment_user_download_bytes(user_id, bytes)
            .await
    }
}
