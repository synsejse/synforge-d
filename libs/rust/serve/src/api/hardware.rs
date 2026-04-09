use axum::Json;
use axum::extract::State;
use synforge_core::api::ServerHardwareResponse;

use crate::{AppError, AppState};

#[utoipa::path(
    get,
    path = "/api/v1/system/hardware",
    tag = "Statistics",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get server hardware limits", body = ServerHardwareResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_server_hardware(
    State(state): State<AppState>,
) -> Result<Json<ServerHardwareResponse>, AppError> {
    Ok(Json(state.service.get_server_hardware().await?))
}
