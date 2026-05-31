use super::{AppError, AppState};
use axum::Json;
use axum::Router;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use synforge_core::api::{
    ChangePasswordRequest, CreateUserRequest, PaginationQuery, UpdateUserRequest, UserListResponse,
    UserMetricsResponse, UserResponse,
};
use synforge_core::model::UserAccount;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/{id}",
            get(get_user_metrics).put(update_user).delete(delete_user),
        )
        .route(
            "/users/{id}/password",
            axum::routing::post(change_user_password),
        )
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    params(PaginationQuery),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "List users", body = UserListResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<UserListResponse>, AppError> {
    Ok(Json(
        state.service.list_users(query.limit, query.offset).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "Users",
    request_body = CreateUserRequest,
    security(("session_auth" = [])),
    responses(
        (status = 201, description = "Create user", body = UserResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 409, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let response = state.service.create_user(request).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    put,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(
        ("id" = Uuid, Path, description = "User identifier")
    ),
    request_body = UpdateUserRequest,
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Update user", body = UserResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(state.service.update_user(id, request).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/users/{id}/password",
    tag = "Users",
    params(
        ("id" = Uuid, Path, description = "User identifier")
    ),
    request_body = ChangePasswordRequest,
    security(("session_auth" = [])),
    responses(
        (status = 204, description = "Change user password"),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn change_user_password(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
    state.service.change_user_password(id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(
        ("id" = Uuid, Path, description = "User identifier")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 204, description = "User deleted"),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError),
        (status = 409, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn delete_user(
    Extension(current_user): Extension<UserAccount>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if current_user.id == id {
        return Err(AppError::from(anyhow::anyhow!(
            synforge_core::error::SynforgeError::Conflict(
                "cannot delete the currently authenticated user".to_string()
            )
        )));
    }
    state.service.delete_user(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(
        ("id" = Uuid, Path, description = "User identifier")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get user metrics", body = UserMetricsResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn get_user_metrics(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserMetricsResponse>, AppError> {
    Ok(Json(state.service.get_user_metrics(id).await?))
}
