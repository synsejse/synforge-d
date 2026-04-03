use std::path::{Component, PathBuf};

use axum::extract::{Extension, Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use axum::routing::{get, post};
use synforge_core::api::{
    BrowseRepositoryRequest, BrowseRepositoryResponse, BuildJobListResponse, BuildJobResponse,
    ChangePasswordRequest, ConfigSchemaResponse, CreatePackageRequest, CreateUserRequest,
    EffectiveConfigDto, JobListQuery, LogChunkQuery, LogChunkResponse, LogManifestResponse,
    LogMetaResponse, MockChrootListResponse, PackageBuildHistoryResponse, PackageListQuery,
    PackageListResponse, PackageRepoFilesResponse, PackageResponse, RepoInventoryQuery,
    PruneJobsResponse, RefreshRequest, RebuildRequest, RepoInventoryResponse, RepoSummaryResponse,
    SessionLoginRequest, SessionResponse, SetupInitializeRequest, SetupStatusResponse,
    UpdatePackageRequest, UpdateRuntimeSettingsRequest, UpdateUserRequest, UserListResponse,
    UserMetricsResponse, UserResponse,
};
use synforge_core::model::{UserAccount, UserPermission};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{AppError, AppState, clear_session_cookie, create_session_cookie};

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
        .route("/packages/{name}/repo-files", get(get_package_repo_files))
        .route("/packages/{name}/rebuild", post(trigger_rebuild))
        .route("/packages/{name}/refresh", post(trigger_refresh))
        .route("/jobs", get(list_jobs))
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
        .route("/jobs/{id}/logs", get(get_job_log_manifest))
        .route("/jobs/{id}/logs/meta", get(get_job_log_meta))
        .route("/jobs/{id}/logs/stream", get(get_job_log_chunk))
        .route("/jobs/{id}/artifacts/{*path}", get(download_job_artifact))
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
        ("name" = String, Path, description = "Package name")
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
        ("name" = String, Path, description = "Package name")
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
) -> Result<Json<PackageBuildHistoryResponse>, AppError> {
    Ok(Json(state.service.get_package_build_history(&name).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/packages/{name}/repo-files",
    tag = "Repository",
    params(
        ("name" = String, Path, description = "Package name")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Repository files owned by the package", body = PackageRepoFilesResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_package_repo_files(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<PackageRepoFilesResponse>, AppError> {
    Ok(Json(state.service.get_package_repo_files(&name).await?))
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
        (status = 200, description = "Queue rebuild", body = BuildJobResponse),
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
) -> Result<Json<BuildJobResponse>, AppError> {
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
        (status = 200, description = "Queue refresh", body = BuildJobResponse),
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
) -> Result<Json<BuildJobResponse>, AppError> {
    Ok(Json(state.service.trigger_refresh(&name, request).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs",
    tag = "Jobs",
    params(JobListQuery),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "List jobs", body = BuildJobListResponse),
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
    path = "/api/v1/session",
    tag = "Session",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get current UI session", body = SessionResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_session(
    Extension(user): Extension<UserAccount>,
    State(state): State<AppState>,
) -> Result<Json<SessionResponse>, AppError> {
    Ok(Json(state.service.get_session(user).await))
}

#[utoipa::path(
    get,
    path = "/api/v1/setup/status",
    tag = "Setup",
    responses(
        (status = 200, description = "Get daemon setup status", body = SetupStatusResponse)
    )
)]
pub(crate) async fn get_setup_status(State(state): State<AppState>) -> Result<Json<SetupStatusResponse>, AppError> {
    let initialized = synforge_core::config::DaemonConfig::load_from_file(&state.config_path)
        .map(|config| config.bootstrap_completed)
        .unwrap_or(false);
    Ok(Json(SetupStatusResponse { initialized }))
}

#[utoipa::path(
    post,
    path = "/api/v1/setup/initialize",
    tag = "Setup",
    request_body = SetupInitializeRequest,
    responses(
        (status = 200, description = "Initialize daemon configuration and first admin", body = EffectiveConfigDto),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 409, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn initialize_setup(
    State(state): State<AppState>,
    Json(request): Json<SetupInitializeRequest>,
) -> Result<Json<EffectiveConfigDto>, AppError> {
    Ok(Json(state.service.initialize_setup(request).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/session/login",
    tag = "Session",
    request_body = SessionLoginRequest,
    responses(
        (status = 200, description = "Authenticate and create a UI session", body = SessionResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 503, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn login_session(
    State(state): State<AppState>,
    Json(request): Json<SessionLoginRequest>,
) -> Result<Response, AppError> {
    let user = state
        .service
        .authenticate_user(&request.handle, &request.password, UserPermission::Read)
        .await?;
    let mut response = Json(state.service.get_session(user.clone()).await).into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        create_session_cookie(
            user.id,
            state.service.config().session_secret.as_bytes(),
            state.service.config().public_base_url.starts_with("https://"),
        )?,
    );
    Ok(response)
}

#[utoipa::path(
    post,
    path = "/api/v1/session/logout",
    tag = "Session",
    responses(
        (status = 204, description = "Clear current UI session")
    )
)]
pub(crate) async fn logout_session(State(state): State<AppState>) -> Result<Response, AppError> {
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        clear_session_cookie(state.service.config().public_base_url.starts_with("https://"))?,
    );
    Ok(response)
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "List users", body = UserListResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn list_users(State(state): State<AppState>) -> Result<Json<UserListResponse>, AppError> {
    Ok(Json(state.service.list_users().await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "Users",
    request_body = CreateUserRequest,
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Create user", body = UserResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 409, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(state.service.create_user(request).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(
        ("id" = Uuid, Path, description = "User identifier")
    ),
    request_body = UpdateUserRequest,
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Update user", body = UserResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(state.service.update_user(id, request).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/users/{id}/password",
    tag = "Users",
    params(
        ("id" = Uuid, Path, description = "User identifier")
    ),
    request_body = ChangePasswordRequest,
    security(("session_auth" = [])),
    responses(
        (status = 204, description = "Change user password"),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn change_user_password(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
    state.service.change_user_password(id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(
        ("id" = Uuid, Path, description = "User identifier")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Delete user", body = UserResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn delete_user(
    Extension(current_user): Extension<UserAccount>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    if current_user.id == id {
        return Err(AppError::from(anyhow::anyhow!(
            "cannot delete the currently authenticated user"
        )));
    }
    Ok(Json(state.service.delete_user(id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(
        ("id" = Uuid, Path, description = "User identifier")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get user metrics", body = UserMetricsResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_user_metrics(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserMetricsResponse>, AppError> {
    Ok(Json(state.service.get_user_metrics(id).await?))
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
    path = "/api/v1/jobs/{id}/logs/stream",
    tag = "Logs",
    params(
        ("id" = Uuid, Path, description = "Job identifier"),
        LogChunkQuery
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Read a log chunk", body = LogChunkResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_job_log_chunk(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<LogChunkQuery>,
) -> Result<Json<LogChunkResponse>, AppError> {
    Ok(Json(
        state
            .service
            .get_job_log_chunk(id, query.source, query.cursor, query.offset, query.limit)
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/{id}/logs/meta",
    tag = "Logs",
    params(
        ("id" = Uuid, Path, description = "Job identifier"),
        LogChunkQuery
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get log file metadata", body = LogMetaResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_job_log_meta(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<LogChunkQuery>,
) -> Result<Json<LogMetaResponse>, AppError> {
    Ok(Json(
        state
            .service
            .get_job_log_meta(id, query.source)
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/{id}/artifacts/{path}",
    tag = "Jobs",
    params(
        ("id" = Uuid, Path, description = "Job identifier"),
        ("path" = String, Path, description = "Artifact path relative to the job artifact root")
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
    Path((id, path)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, AppError> {
    let relative_repo_path = normalize_artifact_path(&path)?;
    let artifact_path = state
        .service
        .resolve_job_artifact_path(id, &relative_repo_path)
        .await?;
    let file = tokio::fs::File::open(&artifact_path).await?;
    let file_name = artifact_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow::anyhow!("artifact path {} has no filename", artifact_path.display()))?;

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", file_name))?,
    );

    Ok((headers, axum::body::Body::from_stream(ReaderStream::new(file))))
}

fn normalize_artifact_path(path: &str) -> anyhow::Result<String> {
    let trimmed = path.trim_start_matches('/');
    let normalized = PathBuf::from(trimmed);
    if normalized.as_os_str().is_empty() {
        anyhow::bail!("artifact path must not be empty");
    }

    if normalized.components().any(|component| {
        !matches!(component, Component::Normal(_))
    }) {
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
