use super::{AppError, AppState};
use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use synforge_core::api::{
    BuildJobListResponse, BuildJobResponse, JobListQuery, JobListScope,
    JobResourceUsageListResponse, JobResourceUsageResponse, PruneJobsResponse, TimeSeriesQuery,
    TimeSeriesResponse,
};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/jobs", get(list_jobs))
        .route("/jobs/usage", get(list_job_usage))
        .route("/jobs/timeseries", get(get_jobs_timeseries))
        .route("/jobs/prune-failed", post(prune_failed_jobs))
        .route("/jobs/{id}/kill", post(kill_job))
        .route("/jobs/{id}/retry", post(retry_job))
        .route("/jobs/{id}", get(get_job).delete(delete_job))
        .route("/jobs/{id}/usage", get(get_job_usage))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/timeseries",
    tag = "Jobs",
    params(TimeSeriesQuery),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Bucketed completed-job counts over time", body = TimeSeriesResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn get_jobs_timeseries(
    State(state): State<AppState>,
    Query(query): Query<TimeSeriesQuery>,
) -> Result<Json<TimeSeriesResponse>, AppError> {
    Ok(Json(state.service.get_jobs_timeseries(query.range).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs",
    tag = "Jobs",
    params(JobListQuery),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "List all jobs", body = BuildJobListResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn list_jobs(
    State(state): State<AppState>,
    Query(query): Query<JobListQuery>,
) -> Result<Json<BuildJobListResponse>, AppError> {
    let scope = query.scope.unwrap_or(JobListScope::All);
    let include_deleted = query.include_deleted.unwrap_or(false);
    let response = match scope {
        JobListScope::All => {
            state
                .service
                .list_jobs(
                    query.limit,
                    query.offset,
                    query.status,
                    query.package_name,
                    query.mock_chroot,
                    include_deleted,
                )
                .await?
        }
        JobListScope::Active => {
            state
                .service
                .list_active_jobs(
                    query.limit,
                    query.offset,
                    query.package_name,
                    query.mock_chroot,
                )
                .await?
        }
        JobListScope::Completed => {
            state
                .service
                .list_completed_jobs(
                    query.limit,
                    query.offset,
                    query.status,
                    query.package_name,
                    query.mock_chroot,
                    include_deleted,
                )
                .await?
        }
    };
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/{id}",
    tag = "Jobs",
    params(
        ("id" = Uuid, Path, description = "Job identifier")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get job", body = BuildJobResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BuildJobResponse>, AppError> {
    Ok(Json(state.service.get_job(id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/usage",
    tag = "Jobs",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "List live usage for active jobs", body = JobResourceUsageListResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn list_job_usage(
    State(state): State<AppState>,
) -> Result<Json<JobResourceUsageListResponse>, AppError> {
    Ok(Json(state.service.list_job_resource_usage().await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/{id}/usage",
    tag = "Jobs",
    params(
        ("id" = Uuid, Path, description = "Job identifier")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get live usage for one job", body = JobResourceUsageResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn get_job_usage(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<JobResourceUsageResponse>, AppError> {
    Ok(Json(state.service.get_job_resource_usage(id).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/jobs/{id}/kill",
    tag = "Jobs",
    params(
        ("id" = Uuid, Path, description = "Job identifier")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Kill active job", body = BuildJobResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError),
        (status = 409, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn kill_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BuildJobResponse>, AppError> {
    Ok(Json(state.service.kill_job(id).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/jobs/{id}/retry",
    tag = "Jobs",
    params(
        ("id" = Uuid, Path, description = "Job identifier")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Retry a finished job", body = BuildJobResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError),
        (status = 409, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn retry_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BuildJobResponse>, AppError> {
    Ok(Json(state.service.retry_job(id).await?))
}

#[utoipa::path(
    delete,
    path = "/api/v1/jobs/{id}",
    tag = "Jobs",
    params(
        ("id" = Uuid, Path, description = "Job identifier")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Delete job", body = BuildJobResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn delete_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BuildJobResponse>, AppError> {
    Ok(Json(state.service.delete_job(id).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/jobs/prune-failed",
    tag = "Jobs",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Prune failed and timed out jobs", body = PruneJobsResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn prune_failed_jobs(
    State(state): State<AppState>,
) -> Result<Json<PruneJobsResponse>, AppError> {
    Ok(Json(state.service.prune_failed_jobs().await?))
}
