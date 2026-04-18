use super::{AppError, AppState};
use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use synforge_core::api::{RepoInventoryQuery, RepoInventoryResponse, RepoSummaryResponse};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/repo/files", get(get_repo_inventory))
        .route("/repo/summary", get(get_repo_summary))
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
pub(super) async fn get_repo_inventory(
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
pub(super) async fn get_repo_summary(
    State(state): State<AppState>,
) -> Result<Json<RepoSummaryResponse>, AppError> {
    Ok(Json(state.service.get_repo_summary().await?))
}
