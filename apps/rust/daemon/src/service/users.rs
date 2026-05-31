use synforge_core::{
    api::{
        ChangePasswordRequest, CreateUserRequest, SessionResponse, UpdateUserRequest,
        UserListResponse, UserMetricsResponse, UserResponse, normalize_pagination,
    },
    error::SynforgeError,
    model::{UserAccount, UserPermission},
};
use synforge_database::accounts::AccountService;
use synforge_state::UiSessionRecord;
use uuid::Uuid;

use super::SynforgeService;

impl SynforgeService {
    pub(crate) async fn is_bootstrap_admin_user(&self, user_id: Uuid) -> anyhow::Result<bool> {
        self.account_service()
            .is_bootstrap_admin_user(user_id)
            .await
    }

    pub async fn authenticate_user(
        &self,
        handle: &str,
        password: &str,
        required: UserPermission,
    ) -> anyhow::Result<UserAccount> {
        self.account_service()
            .authenticate_user(handle, password, required)
            .await
    }

    pub async fn authorize_user(
        &self,
        user_id: Uuid,
        required: UserPermission,
    ) -> anyhow::Result<UserAccount> {
        self.account_service()
            .authorize_user(user_id, required)
            .await
    }

    pub async fn authorize_session(
        &self,
        token: &str,
        required: UserPermission,
    ) -> anyhow::Result<UserAccount> {
        let Some(session) = self.runtime_cache.get_ui_session(token).await? else {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        };
        self.authorize_user(session.user_id, required).await
    }

    pub async fn create_ui_session(&self, user_id: Uuid) -> anyhow::Result<UiSessionRecord> {
        self.runtime_cache.create_ui_session(user_id).await
    }

    pub async fn delete_ui_session(&self, token: &str) -> anyhow::Result<()> {
        self.runtime_cache.delete_ui_session(token).await
    }

    pub async fn get_session(&self, user: UserAccount) -> SessionResponse {
        SessionResponse { user }
    }

    pub async fn list_users(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> anyhow::Result<UserListResponse> {
        let (limit, offset) = normalize_pagination(limit, offset);
        self.account_service().list_users(limit, offset).await
    }

    pub async fn create_user(&self, request: CreateUserRequest) -> anyhow::Result<UserResponse> {
        self.account_service().create_user(request).await
    }

    pub async fn bootstrap_admin(
        &self,
        handle: &str,
        display_name: &str,
        password: &str,
    ) -> anyhow::Result<UserResponse> {
        self.account_service()
            .bootstrap_admin(handle, display_name, password)
            .await
    }

    pub async fn update_user(
        &self,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> anyhow::Result<UserResponse> {
        self.account_service().update_user(user_id, request).await
    }

    pub async fn change_user_password(
        &self,
        user_id: Uuid,
        request: ChangePasswordRequest,
    ) -> anyhow::Result<()> {
        self.account_service()
            .change_user_password(user_id, request)
            .await
    }

    pub async fn delete_user(&self, user_id: Uuid) -> anyhow::Result<UserResponse> {
        self.account_service().delete_user(user_id).await
    }

    pub async fn get_user_metrics(&self, user_id: Uuid) -> anyhow::Result<UserMetricsResponse> {
        self.account_service().get_user_metrics(user_id).await
    }

    pub async fn increment_user_download_bytes(
        &self,
        user_id: Uuid,
        bytes: u64,
    ) -> anyhow::Result<()> {
        self.account_service()
            .increment_user_download_bytes(user_id, bytes)
            .await
    }

    fn account_service(&self) -> AccountService {
        AccountService::new(self.store.clone())
    }
}
