use axum::Json;
use axum::extract::State;
use synforge_core::api::CacheStatsResponse;

use crate::{AppError, AppState};

#[utoipa::path(
    get,
    path = "/api/v1/cache/stats",
    tag = "Statistics",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get cache telemetry stats", body = CacheStatsResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_cache_stats(
    State(state): State<AppState>,
) -> Result<Json<CacheStatsResponse>, AppError> {
    Ok(Json(state.service.get_cache_stats().await?))
}
