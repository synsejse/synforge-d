use crate::http::{AppError, AppState};
use axum::Json;
use axum::extract::{Path, Query, State};
use synforge_core::api::{
    BrowseRepositoryRequest, BrowseRepositoryResponse, MockChrootListResponse,
    PackageBuildHistoryResponse, PaginationQuery,
};

#[utoipa::path(
    get,
    path = "/api/v1/mock/chroots",
    tag = "Packages",
    params(PaginationQuery),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "List available mock chroots", body = MockChrootListResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn list_mock_chroots(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<MockChrootListResponse>, AppError> {
    Ok(Json(
        state
            .service
            .list_mock_chroots(query.limit, query.offset)
            .await?,
    ))
}

/// Browse a remote repository's files. Modeled as POST (not GET) because it
/// takes a `repo_url` request body rather than encoding the URL in the path.
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
    let include_deleted = query.include_deleted.unwrap_or(false);
    Ok(Json(
        state
            .service
            .get_package_build_history(&name, query.limit, query.offset, include_deleted)
            .await?,
    ))
}
