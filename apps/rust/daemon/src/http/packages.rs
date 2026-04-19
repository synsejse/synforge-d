use super::{AppError, AppState};
use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use synforge_core::api::{
    BrowseRepositoryRequest, BrowseRepositoryResponse, CreatePackageRequest,
    MockChrootListResponse, PackageActionResponse, PackageActionTargetResult,
    PackageBuildHistoryResponse, PackageListQuery, PackageListResponse, PackageResponse,
    PaginationQuery, RebuildRequest, RefreshAllPackagesProgressResponse,
    RefreshAllPackagesResponse, RefreshRequest, UpdatePackageRequest,
};

pub fn router() -> Router<AppState> {
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
        .route("/packages/refresh-all", post(trigger_refresh_all_packages))
        .route(
            "/packages/refresh-all/progress",
            get(get_refresh_all_packages_progress),
        )
        .route(
            "/packages/{name}/targets/{mock_chroot}/rebuild",
            post(trigger_target_rebuild),
        )
        .route(
            "/packages/{name}/targets/{mock_chroot}/refresh",
            post(trigger_target_refresh),
        )
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
pub(super) async fn list_packages(
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
pub(super) async fn create_package(
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
pub(super) async fn list_mock_chroots(
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
pub(super) async fn browse_repository(
    State(state): State<AppState>,
    Json(request): Json<BrowseRepositoryRequest>,
) -> Result<Json<BrowseRepositoryResponse>, AppError> {
    Ok(Json(state.service.browse_repository(request).await?))
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
pub(super) async fn get_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<PackageResponse>, AppError> {
    Ok(Json(state.service.get_package(&name).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/packages/{name}",
    tag = "Packages",
    params(("name" = String, Path, description = "Package name")),
    request_body = UpdatePackageRequest,
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Update package", body = PackageResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn update_package(
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
    params(("name" = String, Path, description = "Package name")),
    security(("session_auth" = [])),
    responses(
        (status = 204, description = "Package deleted"),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn delete_package(
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
pub(super) async fn get_package_builds(
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
    params(("name" = String, Path, description = "Package name")),
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
pub(super) async fn trigger_rebuild(
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
    params(("name" = String, Path, description = "Package name")),
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
pub(super) async fn trigger_refresh(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<RefreshRequest>,
) -> Result<Json<PackageActionResponse>, AppError> {
    Ok(Json(state.service.trigger_refresh(&name, request).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/packages/refresh-all",
    tag = "Packages",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Queue refresh for all enabled packages", body = RefreshAllPackagesResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 409, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn trigger_refresh_all_packages(
    State(state): State<AppState>,
) -> Result<Json<RefreshAllPackagesResponse>, AppError> {
    Ok(Json(state.service.trigger_refresh_all_packages().await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/packages/refresh-all/progress",
    tag = "Packages",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get latest refresh-all progress", body = RefreshAllPackagesProgressResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn get_refresh_all_packages_progress(
    State(state): State<AppState>,
) -> Result<Json<RefreshAllPackagesProgressResponse>, AppError> {
    Ok(Json(
        state.service.get_refresh_all_packages_progress().await?,
    ))
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
pub(super) async fn trigger_target_rebuild(
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
pub(super) async fn trigger_target_refresh(
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
