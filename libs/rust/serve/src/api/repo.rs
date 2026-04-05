use axum::Json;
use axum::extract::{Query, State};
use synforge_core::api::{
    BrowseRepositoryRequest, BrowseRepositoryResponse, RepoInventoryQuery, RepoInventoryResponse,
    RepoSummaryResponse,
};

use crate::{AppError, AppState};

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
