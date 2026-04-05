use std::path::{Component, PathBuf};

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use synforge_core::api::{
    BuildJobListResponse, BuildJobResponse, JobArtifactListResponse, JobArtifactMetaResponse,
    JobListQuery, PruneJobsResponse,
};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{AppError, AppState};

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
pub(crate) async fn list_jobs(
    State(state): State<AppState>,
    Query(query): Query<JobListQuery>,
) -> Result<Json<BuildJobListResponse>, AppError> {
    Ok(Json(
        state
            .service
            .list_jobs(
                query.limit,
                query.offset,
                query.status,
                query.package_name,
                query.mock_chroot,
            )
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/completed",
    tag = "Jobs",
    params(JobListQuery),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "List completed jobs", body = BuildJobListResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn list_completed_jobs(
    State(state): State<AppState>,
    Query(query): Query<JobListQuery>,
) -> Result<Json<BuildJobListResponse>, AppError> {
    Ok(Json(
        state
            .service
            .list_completed_jobs(
                query.limit,
                query.offset,
                query.status,
                query.package_name,
                query.mock_chroot,
            )
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/active",
    tag = "Jobs",
    params(
        ("limit" = Option<usize>, Query, description = "Maximum number of jobs to return"),
        ("offset" = Option<usize>, Query, description = "Offset for pagination"),
        ("package_name" = Option<String>, Query, description = "Filter by package name"),
        ("mock_chroot" = Option<String>, Query, description = "Filter by target")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "List active jobs", body = BuildJobListResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn list_active_jobs(
    State(state): State<AppState>,
    Query(query): Query<JobListQuery>,
) -> Result<Json<BuildJobListResponse>, AppError> {
    Ok(Json(
        state
            .service
            .list_active_jobs(
                query.limit,
                query.offset,
                query.package_name,
                query.mock_chroot,
            )
            .await?,
    ))
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
pub(crate) async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BuildJobResponse>, AppError> {
    Ok(Json(state.service.get_job(id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/{id}/artifacts",
    tag = "Jobs",
    params(
        ("id" = Uuid, Path, description = "Job identifier")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "List job artifacts", body = JobArtifactListResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn list_job_artifacts(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<JobArtifactListResponse>, AppError> {
    Ok(Json(state.service.get_job_artifacts(id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/{id}/artifacts/{file}/meta",
    tag = "Jobs",
    params(
        ("id" = Uuid, Path, description = "Job identifier"),
        ("file" = String, Path, description = "Artifact file name")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get artifact metadata", body = JobArtifactMetaResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_job_artifact_meta(
    State(state): State<AppState>,
    Path((id, file)): Path<(Uuid, String)>,
) -> Result<Json<JobArtifactMetaResponse>, AppError> {
    Ok(Json(state.service.get_job_artifact_meta(id, &file).await?))
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
pub(crate) async fn delete_job(
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
pub(crate) async fn prune_failed_jobs(
    State(state): State<AppState>,
) -> Result<Json<PruneJobsResponse>, AppError> {
    Ok(Json(state.service.prune_failed_jobs().await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/{id}/artifacts/{file}",
    tag = "Jobs",
    params(
        ("id" = Uuid, Path, description = "Job identifier"),
        ("file" = String, Path, description = "Artifact file name")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Download job artifact", content_type = "application/octet-stream", body = String),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn download_job_artifact(
    State(state): State<AppState>,
    Path((id, file)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, AppError> {
    let relative_artifact_path = normalize_artifact_path(&file)?;
    let artifact_path = state
        .service
        .resolve_job_artifact_path(id, &relative_artifact_path)
        .await?;
    let file = tokio::fs::File::open(&artifact_path).await?;
    let file_name = artifact_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            anyhow::anyhow!("artifact path {} has no filename", artifact_path.display())
        })?;

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", file_name))?,
    );

    Ok((
        headers,
        axum::body::Body::from_stream(ReaderStream::new(file)),
    ))
}

fn normalize_artifact_path(path: &str) -> anyhow::Result<String> {
    let trimmed = path.trim_start_matches('/');
    let normalized = PathBuf::from(trimmed);
    if normalized.as_os_str().is_empty() {
        anyhow::bail!("artifact path must not be empty");
    }

    if normalized
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        anyhow::bail!("artifact path contains invalid components");
    }

    Ok(normalized.to_string_lossy().into_owned())
}
