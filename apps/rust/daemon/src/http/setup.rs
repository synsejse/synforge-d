use super::{AppError, AppState};
use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::{get, post};
use synforge_core::api::{EffectiveConfigDto, SetupInitializeRequest, SetupStatusResponse};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/setup/status", get(get_setup_status))
        .route("/setup/initialize", post(initialize_setup))
}

#[utoipa::path(
    get,
    path = "/api/v1/setup/status",
    tag = "Setup",
    responses(
        (status = 200, description = "Get daemon setup status", body = SetupStatusResponse)
    )
)]
pub(super) async fn get_setup_status(
    State(state): State<AppState>,
) -> Result<Json<SetupStatusResponse>, AppError> {
    let initialized = state
        .service
        .effective_config()
        .await?
        .config
        .bootstrap_completed;
    Ok(Json(SetupStatusResponse { initialized }))
}

#[utoipa::path(
    post,
    path = "/api/v1/setup/initialize",
    tag = "Setup",
    request_body = SetupInitializeRequest,
    responses(
        (status = 200, description = "Initialize daemon and first admin", body = EffectiveConfigDto),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 409, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn initialize_setup(
    State(state): State<AppState>,
    Json(request): Json<SetupInitializeRequest>,
) -> Result<Json<EffectiveConfigDto>, AppError> {
    Ok(Json(state.service.initialize_setup(request).await?))
}
