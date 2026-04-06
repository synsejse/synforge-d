use axum::Json;
use axum::extract::{Path, Query, State};
use synforge_core::api::{
    PackageSyncOperationListQuery, SyncMetricsResponse, SyncOperationListQuery,
    SyncOperationListResponse,
};

use crate::{AppError, AppState};

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
pub(crate) async fn list_sync_operations(
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
pub(crate) async fn list_package_sync_operations(
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
    path = "/api/v1/sync/metrics",
    tag = "Sync",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get sync metrics", body = SyncMetricsResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_sync_metrics(
    State(state): State<AppState>,
) -> Result<Json<SyncMetricsResponse>, AppError> {
    Ok(Json(state.service.get_sync_metrics().await?))
}
