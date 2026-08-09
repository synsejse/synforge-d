use crate::http::{AppError, AppState};
use axum::extract::{Path, State};
use axum::{Json, http::StatusCode};
use synforge_core::api::{RebuildRequest, RefreshRequest, SyncEnqueueResponse};

#[utoipa::path(
    post,
    path = "/api/v1/packages/{name}/rebuild",
    tag = "Packages",
    params(("name" = String, Path, description = "Package name")),
    request_body = RebuildRequest,
    security(("session_auth" = [])),
    responses(
        (status = 202, description = "Queue rebuild sync", body = SyncEnqueueResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError),
        (status = 409, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn trigger_rebuild(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<RebuildRequest>,
) -> Result<(StatusCode, Json<SyncEnqueueResponse>), AppError> {
    Ok((
        StatusCode::ACCEPTED,
        Json(state.service.trigger_rebuild(&name, request).await?),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/packages/{name}/refresh",
    tag = "Packages",
    params(("name" = String, Path, description = "Package name")),
    request_body = RefreshRequest,
    security(("session_auth" = [])),
    responses(
        (status = 202, description = "Queue refresh sync", body = SyncEnqueueResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError),
        (status = 409, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn trigger_refresh(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<RefreshRequest>,
) -> Result<(StatusCode, Json<SyncEnqueueResponse>), AppError> {
    Ok((
        StatusCode::ACCEPTED,
        Json(state.service.trigger_refresh(&name, request).await?),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/packages/{name}/targets/{mock_chroot}/rebuild",
    tag = "Packages",
    params(
        ("name" = String, Path, description = "Package name"),
        ("mock_chroot" = String, Path, description = "Mock chroot target")
    ),
    request_body = RebuildRequest,
    security(("session_auth" = [])),
    responses(
        (status = 202, description = "Queue rebuild sync for one target", body = SyncEnqueueResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError),
        (status = 409, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn trigger_target_rebuild(
    State(state): State<AppState>,
    Path((name, mock_chroot)): Path<(String, String)>,
    Json(request): Json<RebuildRequest>,
) -> Result<(StatusCode, Json<SyncEnqueueResponse>), AppError> {
    Ok((
        StatusCode::ACCEPTED,
        Json(
            state
                .service
                .trigger_target_rebuild(&name, &mock_chroot, request)
                .await?,
        ),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/packages/{name}/targets/{mock_chroot}/refresh",
    tag = "Packages",
    params(
        ("name" = String, Path, description = "Package name"),
        ("mock_chroot" = String, Path, description = "Mock chroot target")
    ),
    request_body = RefreshRequest,
    security(("session_auth" = [])),
    responses(
        (status = 202, description = "Queue refresh sync for one target", body = SyncEnqueueResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError),
        (status = 409, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn trigger_target_refresh(
    State(state): State<AppState>,
    Path((name, mock_chroot)): Path<(String, String)>,
    Json(request): Json<RefreshRequest>,
) -> Result<(StatusCode, Json<SyncEnqueueResponse>), AppError> {
    Ok((
        StatusCode::ACCEPTED,
        Json(
            state
                .service
                .trigger_target_refresh(&name, &mock_chroot, request)
                .await?,
        ),
    ))
}
