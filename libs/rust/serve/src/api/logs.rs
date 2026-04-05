use axum::Json;
use axum::extract::{Path, Query, State};
use synforge_core::api::{LogChunkQuery, LogChunkResponse, LogManifestResponse, LogMetaResponse};
use uuid::Uuid;

use crate::{AppError, AppState};

#[utoipa::path(
    get,
    path = "/api/v1/jobs/{id}/logs",
    tag = "Logs",
    params(
        ("id" = Uuid, Path, description = "Job identifier")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get available log sources for a job", body = LogManifestResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_job_log_manifest(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<LogManifestResponse>, AppError> {
    Ok(Json(state.service.get_job_log_manifest(id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/{id}/logs/{source}/stream",
    tag = "Logs",
    params(
        ("id" = Uuid, Path, description = "Job identifier"),
        ("source" = String, Path, description = "Log source path"),
        ("cursor" = Option<u64>, Query, description = "Current byte cursor"),
        ("offset" = Option<i64>, Query, description = "Relative byte offset from the cursor"),
        ("limit" = Option<usize>, Query, description = "Maximum bytes to read")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Read a log chunk", body = LogChunkResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_job_log_chunk_by_source(
    State(state): State<AppState>,
    Path((id, source)): Path<(Uuid, String)>,
    Query(query): Query<LogChunkQuery>,
) -> Result<Json<LogChunkResponse>, AppError> {
    Ok(Json(
        state
            .service
            .get_job_log_chunk(id, source, query.cursor, query.offset, query.limit)
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/{id}/logs/{source}/meta",
    tag = "Logs",
    params(
        ("id" = Uuid, Path, description = "Job identifier"),
        ("source" = String, Path, description = "Log source path")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get log file metadata", body = LogMetaResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_job_log_meta_by_source(
    State(state): State<AppState>,
    Path((id, source)): Path<(Uuid, String)>,
) -> Result<Json<LogMetaResponse>, AppError> {
    Ok(Json(state.service.get_job_log_meta(id, source).await?))
}
