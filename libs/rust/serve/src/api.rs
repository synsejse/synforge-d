use std::path::{Component, PathBuf};

use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use synforge_core::api::{
    BrowseRepositoryRequest, BrowseRepositoryResponse, BuildJobListResponse, BuildJobResponse,
    ConfigSchemaResponse, CreatePackageRequest, EffectiveConfigDto, JobArtifactListResponse,
    JobArtifactMetaResponse, JobListQuery, LogChunkQuery, LogChunkResponse, LogManifestResponse,
    LogMetaResponse, MockChrootListResponse, PackageActionResponse, PackageActionTargetResult,
    PackageBuildHistoryResponse, PackageListQuery, PackageListResponse, PackageResponse,
    PaginationQuery, PruneJobsResponse, RebuildRequest, RefreshRequest, RepoInventoryQuery,
    RepoInventoryResponse, RepoSummaryResponse, UpdatePackageRequest, UpdateRuntimeSettingsRequest,
};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{AppError, AppState};

pub(crate) mod session;
pub(crate) mod users;
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
    path = "/api/v1/packages",
    tag = "Packages",
    params(PackageListQuery),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "List packages", body = PackageListResponse),
        (status = 401, description = "Missing or invalid session", body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn list_packages(
    State(state): State<AppState>,
    Query(query): Query<PackageListQuery>,
) -> Result<Json<PackageListResponse>, AppError> {
    Ok(Json(
        state
            .service
            .list_packages(query.limit, query.offset, query.search, query.enabled)
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/packages/{name}",
    tag = "Packages",
    params(
        ("name" = String, Path, description = "Package name"),
        PaginationQuery
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get package", body = PackageResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<PackageResponse>, AppError> {
    Ok(Json(state.service.get_package(&name).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/packages",
    tag = "Packages",
    request_body = CreatePackageRequest,
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Create package", body = PackageResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 409, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn create_package(
    State(state): State<AppState>,
    Json(request): Json<CreatePackageRequest>,
) -> Result<Json<PackageResponse>, AppError> {
    Ok(Json(state.service.create_package(request).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/mock/chroots",
    tag = "Packages",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "List available mock chroots", body = MockChrootListResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn list_mock_chroots(
    State(state): State<AppState>,
) -> Result<Json<MockChrootListResponse>, AppError> {
    Ok(Json(state.service.list_mock_chroots().await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/repositories/browse",
    tag = "Repository",
    request_body = BrowseRepositoryRequest,
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Browse repository files", body = BrowseRepositoryResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn browse_repository(
    State(state): State<AppState>,
    Json(request): Json<BrowseRepositoryRequest>,
) -> Result<Json<BrowseRepositoryResponse>, AppError> {
    Ok(Json(state.service.browse_repository(request).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/packages/{name}",
    tag = "Packages",
    params(
        ("name" = String, Path, description = "Package name")
    ),
    request_body = UpdatePackageRequest,
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Update package", body = PackageResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn update_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<UpdatePackageRequest>,
) -> Result<Json<PackageResponse>, AppError> {
    Ok(Json(state.service.update_package(&name, request).await?))
}

#[utoipa::path(
    delete,
    path = "/api/v1/packages/{name}",
    tag = "Packages",
    params(
        ("name" = String, Path, description = "Package name")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 204, description = "Package deleted"),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn delete_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    state.service.delete_package(&name).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/packages/{name}/builds",
    tag = "Packages",
    params(
        ("name" = String, Path, description = "Package name"),
        PaginationQuery
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Package build history", body = PackageBuildHistoryResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_package_builds(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PackageBuildHistoryResponse>, AppError> {
    Ok(Json(
        state
            .service
            .get_package_build_history(&name, query.limit, query.offset)
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/packages/{name}/rebuild",
    tag = "Packages",
    params(
        ("name" = String, Path, description = "Package name")
    ),
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
    params(
        ("name" = String, Path, description = "Package name")
    ),
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

#[utoipa::path(
    get,
    path = "/api/v1/config/effective",
    tag = "Settings",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get effective daemon configuration", body = EffectiveConfigDto),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_effective_config(
    State(state): State<AppState>,
) -> Result<Json<EffectiveConfigDto>, AppError> {
    Ok(Json(state.service.effective_config().await))
}

#[utoipa::path(
    get,
    path = "/api/v1/config/schema",
    tag = "Settings",
    responses(
        (status = 200, description = "Get editable daemon configuration schema", body = ConfigSchemaResponse)
    )
)]
pub(crate) async fn get_config_schema(
    State(state): State<AppState>,
) -> Result<Json<ConfigSchemaResponse>, AppError> {
    Ok(Json(state.service.config_schema().await))
}

#[utoipa::path(
    post,
    path = "/api/v1/config/runtime",
    tag = "Settings",
    request_body = UpdateRuntimeSettingsRequest,
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Update runtime-editable daemon configuration", body = EffectiveConfigDto),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn update_runtime_settings(
    State(state): State<AppState>,
    Json(request): Json<UpdateRuntimeSettingsRequest>,
) -> Result<Json<EffectiveConfigDto>, AppError> {
    Ok(Json(state.service.update_runtime_settings(request).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/repo/files",
    tag = "Repository",
    params(RepoInventoryQuery),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "List managed repository files", body = RepoInventoryResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_repo_inventory(
    State(state): State<AppState>,
    Query(query): Query<RepoInventoryQuery>,
) -> Result<Json<RepoInventoryResponse>, AppError> {
    Ok(Json(
        state
            .service
            .get_repo_inventory(
                query.limit,
                query.offset,
                query.package_name,
                query.mock_chroot,
                query.kind,
            )
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/repo/summary",
    tag = "Repository",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get repository summary metrics", body = RepoSummaryResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_repo_summary(
    State(state): State<AppState>,
) -> Result<Json<RepoSummaryResponse>, AppError> {
    Ok(Json(state.service.get_repo_summary().await?))
}
