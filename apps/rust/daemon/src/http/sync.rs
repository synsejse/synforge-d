use super::{AppError, AppState};
use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use synforge_core::api::{
    PackageSyncOperationListQuery, SyncMetricsResponse, SyncOperationListQuery,
    SyncOperationListResponse, SyncScheduleQuery, SyncScheduleResponse, TimeSeriesQuery,
    TimeSeriesResponse,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sync/operations", get(list_sync_operations))
        .route("/sync/metrics", get(get_sync_metrics))
        .route("/sync/timeseries", get(get_sync_timeseries))
        .route("/sync/schedule", get(get_sync_schedule))
        .route(
            "/packages/{name}/sync/operations",
            get(list_package_sync_operations),
        )
}

#[utoipa::path(
    get,
    path = "/api/v1/sync/timeseries",
    tag = "Sync",
    params(TimeSeriesQuery),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Bucketed sync-operation counts over time", body = TimeSeriesResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn get_sync_timeseries(
    State(state): State<AppState>,
    Query(query): Query<TimeSeriesQuery>,
) -> Result<Json<TimeSeriesResponse>, AppError> {
    Ok(Json(state.service.get_sync_timeseries(query.range).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/sync/operations",
    tag = "Sync",
    params(SyncOperationListQuery),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "List sync operations", body = SyncOperationListResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn list_sync_operations(
    State(state): State<AppState>,
    Query(query): Query<SyncOperationListQuery>,
) -> Result<Json<SyncOperationListResponse>, AppError> {
    Ok(Json(
        state
            .service
            .list_sync_operations(query.limit, query.offset, query.package_name, query.status)
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/packages/{name}/sync/operations",
    tag = "Sync",
    params(
        ("name" = String, Path, description = "Package name"),
        PackageSyncOperationListQuery
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "List package sync operations", body = SyncOperationListResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn list_package_sync_operations(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(query): Query<PackageSyncOperationListQuery>,
) -> Result<Json<SyncOperationListResponse>, AppError> {
    Ok(Json(
        state
            .service
            .list_package_sync_operations(&name, query.limit, query.offset, query.status)
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/sync/schedule",
    tag = "Sync",
    params(SyncScheduleQuery),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Upcoming poll schedule", body = SyncScheduleResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn get_sync_schedule(
    State(state): State<AppState>,
    Query(query): Query<SyncScheduleQuery>,
) -> Result<Json<SyncScheduleResponse>, AppError> {
    Ok(Json(state.service.get_sync_schedule(query.limit).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/sync/metrics",
    tag = "Sync",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get sync metrics", body = SyncMetricsResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn get_sync_metrics(
    State(state): State<AppState>,
) -> Result<Json<SyncMetricsResponse>, AppError> {
    Ok(Json(state.service.get_sync_metrics().await?))
}
