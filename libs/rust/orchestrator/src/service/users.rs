use synforge_core::{
    api::{
        ChangePasswordRequest, CreateUserRequest, SessionResponse, UpdateUserRequest,
        UserListResponse, UserMetricsResponse, UserResponse,
    },
    error::SynforgeError,
    model::{UserAccount, UserPermission},
};
use uuid::Uuid;

use super::SynforgeService;
use crate::auth::{hash_password, verify_password};
use crate::db::UserStore;

impl SynforgeService {
    pub async fn authenticate_user(
        &self,
        handle: &str,
        password: &str,
        required: UserPermission,
    ) -> anyhow::Result<UserAccount> {
        let Some(record) = self.store.get_user_auth_by_handle(handle).await? else {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        };
        if !record.user.active {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        }
        if !verify_password(&record.password_hash, password)? {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        }
        if !record.user.has_permission(required) {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        }
        Ok(record.user)
    }

    pub async fn authorize_user(
        &self,
        user_id: Uuid,
        required: UserPermission,
    ) -> anyhow::Result<UserAccount> {
        let Some(summary) = self.store.get_user(user_id).await? else {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        };
        if !summary.user.active || !summary.user.has_permission(required) {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        }
        Ok(summary.user)
    }

    pub async fn get_session(&self, user: UserAccount) -> SessionResponse {
        SessionResponse { user }
    }

    pub async fn list_users(&self) -> anyhow::Result<UserListResponse> {
        let users = self
            .store
            .list_users()
            .await?
            .into_iter()
            .map(|summary| UserResponse {
                user: summary.user,
                metrics: summary.metrics,
            })
            .collect();
        Ok(UserListResponse { users })
    }

    pub async fn create_user(&self, request: CreateUserRequest) -> anyhow::Result<UserResponse> {
        super::validate_user_handle(&request.handle)?;
        super::validate_display_name(&request.display_name)?;
        super::validate_password(&request.password)?;
        super::validate_permissions(&request.permissions)?;
        if self
            .store
            .get_user_by_handle(&request.handle)
            .await?
            .is_some()
        {
            return Err(anyhow::anyhow!(SynforgeError::Conflict(format!(
                "user handle {} already exists",
                request.handle
            ))));
        }
        let password_hash = hash_password(&request.password)?;
        let summary = self
            .store
            .create_user(
                &request.handle,
                &request.display_name,
                &password_hash,
                request.active,
                &request.permissions,
            )
            .await?;
        Ok(UserResponse {
            user: summary.user,
            metrics: summary.metrics,
        })
    }

    pub async fn bootstrap_admin(
        &self,
        handle: &str,
        display_name: &str,
        password: &str,
    ) -> anyhow::Result<UserResponse> {
        super::validate_user_handle(handle)?;
        super::validate_display_name(display_name)?;
        super::validate_password(password)?;
        if self.store.user_count().await? > 0 {
            anyhow::bail!("initial admin already exists");
        }
        let password_hash = hash_password(password)?;
        let summary = self
            .store
            .create_user(
                handle,
                display_name,
                &password_hash,
                true,
                &[
                    UserPermission::Read,
                    UserPermission::Write,
                    UserPermission::Repo,
                ],
            )
            .await?;
        Ok(UserResponse {
            user: summary.user,
            metrics: summary.metrics,
        })
    }

    pub async fn update_user(
        &self,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> anyhow::Result<UserResponse> {
        super::validate_user_handle(&request.handle)?;
        super::validate_display_name(&request.display_name)?;
        super::validate_permissions(&request.permissions)?;
        if let Some(existing) = self.store.get_user_by_handle(&request.handle).await?
            && existing.user.id != user_id
        {
            return Err(anyhow::anyhow!(SynforgeError::Conflict(format!(
                "user handle {} already exists",
                request.handle
            ))));
        }
        let summary = self
            .store
            .update_user(
                user_id,
                &request.handle,
                &request.display_name,
                request.active,
                &request.permissions,
            )
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(user_id.to_string())))?;
        Ok(UserResponse {
            user: summary.user,
            metrics: summary.metrics,
        })
    }

    pub async fn change_user_password(
        &self,
        user_id: Uuid,
        request: ChangePasswordRequest,
    ) -> anyhow::Result<()> {
        super::validate_password(&request.password)?;
        let password_hash = hash_password(&request.password)?;
        let updated = self
            .store
            .update_user_password(user_id, &password_hash)
            .await?;
        if !updated {
            return Err(anyhow::anyhow!(SynforgeError::NotFound(
                user_id.to_string()
            )));
        }
        Ok(())
    }

    pub async fn delete_user(&self, user_id: Uuid) -> anyhow::Result<UserResponse> {
        if self.store.get_user(user_id).await?.is_none() {
            return Err(anyhow::anyhow!(SynforgeError::NotFound(
                user_id.to_string()
            )));
        }
        let user_count = self.store.user_count().await?;
        if user_count <= 1 {
            anyhow::bail!("cannot delete the last user");
        }
        let summary = self
            .store
            .delete_user(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(user_id.to_string())))?;
        Ok(UserResponse {
            user: summary.user,
            metrics: summary.metrics,
        })
    }

    pub async fn get_user_metrics(&self, user_id: Uuid) -> anyhow::Result<UserMetricsResponse> {
        let summary = self
            .store
            .get_user(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(user_id.to_string())))?;
        Ok(UserMetricsResponse {
            metrics: summary.metrics,
        })
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
