use crate::http::{AppError, AppState};
use axum::Json;
use axum::extract::{Path, State};
use synforge_core::api::{
    PackageActionResponse, PackageActionTargetResult, RebuildRequest, RefreshRequest,
};

#[utoipa::path(
    post,
    path = "/api/v1/packages/{name}/rebuild",
    tag = "Packages",
    params(("name" = String, Path, description = "Package name")),
    request_body = RebuildRequest,
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Queue rebuild", body = PackageActionResponse),
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
) -> Result<Json<PackageActionResponse>, AppError> {
    Ok(Json(state.service.trigger_rebuild(&name, request).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/packages/{name}/refresh",
    tag = "Packages",
    params(("name" = String, Path, description = "Package name")),
    request_body = RefreshRequest,
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Queue refresh", body = PackageActionResponse),
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
) -> Result<Json<PackageActionResponse>, AppError> {
    Ok(Json(state.service.trigger_refresh(&name, request).await?))
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
        (status = 200, description = "Queue rebuild for one target", body = PackageActionTargetResult),
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
) -> Result<Json<PackageActionTargetResult>, AppError> {
    Ok(Json(
        state
            .service
            .trigger_target_rebuild(&name, &mock_chroot, request)
            .await?,
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
        (status = 200, description = "Queue refresh for one target", body = PackageActionTargetResult),
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
) -> Result<Json<PackageActionTargetResult>, AppError> {
    Ok(Json(
        state
            .service
            .trigger_target_refresh(&name, &mock_chroot, request)
            .await?,
    ))
}
