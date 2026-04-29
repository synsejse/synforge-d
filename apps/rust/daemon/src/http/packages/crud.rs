use crate::http::{AppError, AppState};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use synforge_core::api::{
    CreatePackageRequest, PackageListQuery, PackageListResponse, PackageResponse, PaginationQuery,
    UpdatePackageRequest,
};

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
    params(("name" = String, Path, description = "Package name")),
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
