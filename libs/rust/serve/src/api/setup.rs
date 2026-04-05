use axum::Json;
use axum::extract::State;
use synforge_core::api::{EffectiveConfigDto, SetupInitializeRequest, SetupStatusResponse};

use crate::{AppError, AppState};

#[utoipa::path(
    get,
    path = "/api/v1/setup/status",
    tag = "Setup",
    responses(
        (status = 200, description = "Get daemon setup status", body = SetupStatusResponse)
    )
)]
pub(crate) async fn get_setup_status(
    State(state): State<AppState>,
) -> Result<Json<SetupStatusResponse>, AppError> {
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
