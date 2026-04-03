use std::path::{Component, PathBuf};

use axum::extract::{Extension, Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use axum::routing::{get, post};
use synforge_core::api::{
    BrowseRepositoryRequest, BrowseRepositoryResponse, BuildJobListResponse, BuildJobResponse,
    ChangePasswordRequest, ConfigSchemaResponse, CreatePackageRequest, CreateUserRequest,
    EffectiveConfigDto, LogChunkQuery, LogChunkResponse, LogManifestResponse, LogMetaResponse,
    MockChrootListResponse, PackageBuildHistoryResponse, PackageListResponse, PaginationQuery,
    PackageRepoFilesResponse, PackageResponse, PruneJobsResponse, RefreshRequest, RebuildRequest,
    RepoInventoryResponse, SessionLoginRequest, SessionResponse, SetupInitializeRequest,
    SetupStatusResponse, UpdatePackageRequest, UpdateRuntimeSettingsRequest, UpdateUserRequest,
    UserListResponse, UserMetricsResponse, UserResponse,
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
        .route("/config/schema", get(get_config_schema))
        .route("/config/effective", get(get_effective_config))
        .route("/config/runtime", post(update_runtime_settings))
}

async fn list_packages(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PackageListResponse>, AppError> {
    Ok(Json(PackageListResponse {
        packages: state.service.list_packages(query.limit, query.offset).await?,
    }))
}

async fn get_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<PackageResponse>, AppError> {
    Ok(Json(state.service.get_package(&name).await?))
}

async fn create_package(
    State(state): State<AppState>,
    Json(request): Json<CreatePackageRequest>,
) -> Result<Json<PackageResponse>, AppError> {
    Ok(Json(state.service.create_package(request).await?))
}

async fn list_mock_chroots(
    State(state): State<AppState>,
) -> Result<Json<MockChrootListResponse>, AppError> {
    Ok(Json(state.service.list_mock_chroots().await?))
}

async fn browse_repository(
    State(state): State<AppState>,
    Json(request): Json<BrowseRepositoryRequest>,
) -> Result<Json<BrowseRepositoryResponse>, AppError> {
    Ok(Json(state.service.browse_repository(request).await?))
}

async fn update_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<UpdatePackageRequest>,
) -> Result<Json<PackageResponse>, AppError> {
    Ok(Json(state.service.update_package(&name, request).await?))
}

async fn delete_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    state.service.delete_package(&name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_package_builds(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<PackageBuildHistoryResponse>, AppError> {
    Ok(Json(state.service.get_package_build_history(&name).await?))
}

async fn get_package_repo_files(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<PackageRepoFilesResponse>, AppError> {
    Ok(Json(state.service.get_package_repo_files(&name).await?))
}

async fn trigger_rebuild(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<RebuildRequest>,
) -> Result<Json<BuildJobResponse>, AppError> {
    Ok(Json(state.service.trigger_rebuild(&name, request).await?))
}

async fn trigger_refresh(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<RefreshRequest>,
) -> Result<Json<BuildJobResponse>, AppError> {
    Ok(Json(state.service.trigger_refresh(&name, request).await?))
}

async fn list_jobs(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<BuildJobListResponse>, AppError> {
    Ok(Json(BuildJobListResponse {
        jobs: state.service.list_jobs(query.limit, query.offset).await?,
    }))
}

async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BuildJobResponse>, AppError> {
    Ok(Json(state.service.get_job(id).await?))
}

async fn get_session(
    Extension(user): Extension<UserAccount>,
    State(state): State<AppState>,
) -> Result<Json<SessionResponse>, AppError> {
    Ok(Json(state.service.get_session(user).await))
}

async fn get_setup_status(State(state): State<AppState>) -> Result<Json<SetupStatusResponse>, AppError> {
    let initialized = synforge_core::config::DaemonConfig::load_from_file(&state.config_path)
        .map(|config| config.bootstrap_completed)
        .unwrap_or(false);
    Ok(Json(SetupStatusResponse { initialized }))
}

async fn initialize_setup(
    State(state): State<AppState>,
    Json(request): Json<SetupInitializeRequest>,
) -> Result<Json<EffectiveConfigDto>, AppError> {
    Ok(Json(state.service.initialize_setup(request).await?))
}

async fn login_session(
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

async fn logout_session(State(state): State<AppState>) -> Result<Response, AppError> {
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        clear_session_cookie(state.service.config().public_base_url.starts_with("https://"))?,
    );
    Ok(response)
}

async fn list_users(State(state): State<AppState>) -> Result<Json<UserListResponse>, AppError> {
    Ok(Json(state.service.list_users().await?))
}

async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(state.service.create_user(request).await?))
}

async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(state.service.update_user(id, request).await?))
}

async fn change_user_password(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
    state.service.change_user_password(id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_user(
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

async fn get_user_metrics(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserMetricsResponse>, AppError> {
    Ok(Json(state.service.get_user_metrics(id).await?))
}

async fn delete_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BuildJobResponse>, AppError> {
    Ok(Json(state.service.delete_job(id).await?))
}

async fn prune_failed_jobs(
    State(state): State<AppState>,
) -> Result<Json<PruneJobsResponse>, AppError> {
    Ok(Json(state.service.prune_failed_jobs().await?))
}

async fn get_job_log_manifest(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<LogManifestResponse>, AppError> {
    Ok(Json(state.service.get_job_log_manifest(id).await?))
}

async fn get_job_log_chunk(
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

async fn get_job_log_meta(
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

async fn download_job_artifact(
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

async fn get_effective_config(
    State(state): State<AppState>,
) -> Result<Json<EffectiveConfigDto>, AppError> {
    Ok(Json(state.service.effective_config().await))
}

async fn get_config_schema(
    State(state): State<AppState>,
) -> Result<Json<ConfigSchemaResponse>, AppError> {
    Ok(Json(state.service.config_schema().await))
}

async fn update_runtime_settings(
    State(state): State<AppState>,
    Json(request): Json<UpdateRuntimeSettingsRequest>,
) -> Result<Json<EffectiveConfigDto>, AppError> {
    Ok(Json(state.service.update_runtime_settings(request).await?))
}

async fn get_repo_inventory(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<RepoInventoryResponse>, AppError> {
    Ok(Json(state.service.get_repo_inventory(query.limit, query.offset).await?))
}
