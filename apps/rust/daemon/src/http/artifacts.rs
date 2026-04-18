use std::path::{Component, PathBuf};

use super::{AppError, AppState};
use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use axum::routing::get;
use synforge_core::api::{JobArtifactListResponse, JobArtifactMetaResponse};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/jobs/{id}/artifacts", get(list_job_artifacts))
        .route(
            "/jobs/{id}/artifacts/{file}/meta",
            get(get_job_artifact_meta),
        )
        .route(
            "/jobs/{id}/artifacts/{file}/content",
            get(download_job_artifact),
        )
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
pub(super) async fn list_job_artifacts(
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
pub(super) async fn get_job_artifact_meta(
    State(state): State<AppState>,
    Path((id, file)): Path<(Uuid, String)>,
) -> Result<Json<JobArtifactMetaResponse>, AppError> {
    Ok(Json(state.service.get_job_artifact_meta(id, &file).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/{id}/artifacts/{file}/content",
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
pub(super) async fn download_job_artifact(
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
