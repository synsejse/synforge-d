//! Postgres-backed account service.

use argon2::password_hash::{Error as PasswordHashError, PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
use synforge_core::{
    api::{
        ChangePasswordRequest, CreateUserRequest, UpdateUserRequest, UserListResponse,
        UserMetricsResponse, UserResponse,
    },
    error::SynforgeError,
    model::{UserAccount, UserPermission},
    validation::{
        validate_display_name, validate_password, validate_permissions, validate_user_handle,
    },
};
use uuid::Uuid;

use crate::{DieselStore, users::PostgresUserStore};

#[derive(Clone)]
pub struct AccountService {
    users: PostgresUserStore,
}

impl AccountService {
    pub fn new(store: DieselStore) -> Self {
        Self {
            users: PostgresUserStore::new(store),
        }
    }

    pub async fn is_bootstrap_admin_user(&self, user_id: Uuid) -> anyhow::Result<bool> {
        let users = self.users.list_users().await?;
        let Some(first_user) = users
            .iter()
            .min_by_key(|summary| summary.user.created_at)
            .map(|summary| &summary.user)
        else {
            return Ok(false);
        };
        Ok(first_user.id == user_id)
    }

    pub async fn authenticate_user(
        &self,
        handle: &str,
        password: &str,
        required: UserPermission,
    ) -> anyhow::Result<UserAccount> {
        let Some(record) = self.users.get_user_auth_by_handle(handle).await? else {
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
        let Some(summary) = self.users.get_user(user_id).await? else {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        };
        if !summary.user.active || !summary.user.has_permission(required) {
            return Err(anyhow::anyhow!(SynforgeError::Unauthorized));
        }
        Ok(summary.user)
    }

    pub async fn list_users(&self) -> anyhow::Result<UserListResponse> {
        let users = self
            .users
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
        validate_user_handle(&request.handle)?;
        validate_display_name(&request.display_name)?;
        validate_password(&request.password)?;
        validate_permissions(&request.permissions)?;
        if self
            .users
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
            .users
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
        validate_user_handle(handle)?;
        validate_display_name(display_name)?;
        validate_password(password)?;
        if self.users.user_count().await? > 0 {
            anyhow::bail!("initial admin already exists");
        }
        let password_hash = hash_password(password)?;
        let summary = self
            .users
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
        validate_user_handle(&request.handle)?;
        validate_display_name(&request.display_name)?;
        validate_permissions(&request.permissions)?;
        if let Some(existing) = self.users.get_user_by_handle(&request.handle).await?
            && existing.user.id != user_id
        {
            return Err(anyhow::anyhow!(SynforgeError::Conflict(format!(
                "user handle {} already exists",
                request.handle
            ))));
        }
        let summary = self
            .users
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
        validate_password(&request.password)?;
        let password_hash = hash_password(&request.password)?;
        let updated = self
            .users
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
        if self.users.get_user(user_id).await?.is_none() {
            return Err(anyhow::anyhow!(SynforgeError::NotFound(
                user_id.to_string()
            )));
        }
        let user_count = self.users.user_count().await?;
        if user_count <= 1 {
            anyhow::bail!("cannot delete the last user");
        }
        let summary = self
            .users
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
            .users
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
        self.users
            .increment_user_download_bytes(user_id, bytes)
            .await
    }
}

fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}

fn verify_password(hash: &str, password: &str) -> anyhow::Result<bool> {
    let parsed = PasswordHash::new(hash).map_err(|error| anyhow::anyhow!(error.to_string()))?;
    match Argon2::default().verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(PasswordHashError::Password) => Ok(false),
        Err(error) => Err(anyhow::anyhow!(error.to_string())),
    }
}
