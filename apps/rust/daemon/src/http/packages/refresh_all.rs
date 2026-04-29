use crate::http::{AppError, AppState};
use axum::Json;
use axum::extract::State;
use synforge_core::api::{RefreshAllPackagesProgressResponse, RefreshAllPackagesResponse};

#[utoipa::path(
    post,
    path = "/api/v1/packages/refresh-all",
    tag = "Packages",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Queue refresh for all enabled packages", body = RefreshAllPackagesResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 409, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn trigger_refresh_all_packages(
    State(state): State<AppState>,
) -> Result<Json<RefreshAllPackagesResponse>, AppError> {
    Ok(Json(state.service.trigger_refresh_all_packages().await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/packages/refresh-all/progress",
    tag = "Packages",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get latest refresh-all progress", body = RefreshAllPackagesProgressResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_refresh_all_packages_progress(
    State(state): State<AppState>,
) -> Result<Json<RefreshAllPackagesProgressResponse>, AppError> {
    Ok(Json(
        state.service.get_refresh_all_packages_progress().await?,
    ))
}
