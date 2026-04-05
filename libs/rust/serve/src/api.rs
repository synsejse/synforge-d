use std::path::{Component, PathBuf};

use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use synforge_core::api::{
    BuildJobListResponse, BuildJobResponse, JobArtifactListResponse, JobArtifactMetaResponse,
    JobListQuery, PruneJobsResponse,
};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{AppError, AppState};

pub(crate) mod config;
pub(crate) mod logs;
pub(crate) mod packages;
pub(crate) mod repo;
pub(crate) mod session;
pub(crate) mod users;
pub(crate) use config::{get_config_schema, get_effective_config, update_runtime_settings};
pub(crate) use logs::{
    get_job_log_chunk_by_source, get_job_log_manifest, get_job_log_meta_by_source,
};
pub(crate) use packages::{
    create_package, delete_package, get_package, get_package_builds, list_mock_chroots,
    list_packages, trigger_rebuild, trigger_refresh, trigger_target_rebuild,
    trigger_target_refresh, update_package,
};
pub(crate) use repo::{browse_repository, get_repo_inventory, get_repo_summary};
pub(crate) use session::{
    get_session, get_setup_status, initialize_setup, login_session, logout_session,
};
pub(crate) use users::{
    change_user_password, create_user, delete_user, get_user_metrics, list_users, update_user,
};

pub fn router(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/packages", get(list_packages).post(create_package))
        .route("/mock/chroots", get(list_mock_chroots))
        .route("/repositories/browse", post(browse_repository))
        .route(
            "/packages/{name}",
            get(get_package).put(update_package).delete(delete_package),
        )
        .route("/packages/{name}/builds", get(get_package_builds))
        .route("/packages/{name}/rebuild", post(trigger_rebuild))
        .route("/packages/{name}/refresh", post(trigger_refresh))
        .route(
            "/packages/{name}/targets/{mock_chroot}/rebuild",
            post(trigger_target_rebuild),
        )
        .route(
            "/packages/{name}/targets/{mock_chroot}/refresh",
            post(trigger_target_refresh),
        )
        .route("/jobs", get(list_jobs))
        .route("/jobs/active", get(list_active_jobs))
        .route("/jobs/completed", get(list_completed_jobs))
        .route("/jobs/prune-failed", post(prune_failed_jobs))
        .route("/session", get(get_session))
        .route("/session/login", post(login_session))
        .route("/session/logout", post(logout_session))
        .route("/setup/status", get(get_setup_status))
        .route("/setup/initialize", post(initialize_setup))
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/{id}",
            get(get_user_metrics).put(update_user).delete(delete_user),
        )
        .route("/users/{id}/password", post(change_user_password))
        .route("/jobs/{id}", get(get_job).delete(delete_job))
        .route("/jobs/{id}/artifacts", get(list_job_artifacts))
        .route(
            "/jobs/{id}/artifacts/{file}/meta",
            get(get_job_artifact_meta),
        )
        .route("/jobs/{id}/logs", get(get_job_log_manifest))
        .route(
            "/jobs/{id}/logs/{source}/meta",
            get(get_job_log_meta_by_source),
        )
        .route(
            "/jobs/{id}/logs/{source}/stream",
            get(get_job_log_chunk_by_source),
        )
        .route("/jobs/{id}/artifacts/{file}", get(download_job_artifact))
        .route("/repo/files", get(get_repo_inventory))
        .route("/repo/summary", get(get_repo_summary))
        .route("/config/schema", get(get_config_schema))
        .route("/config/effective", get(get_effective_config))
        .route("/config/runtime", post(update_runtime_settings))
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
