use super::{AppError, AppState};
use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::get;
use synforge_core::api::ServerHardwareResponse;

pub fn router() -> Router<AppState> {
    Router::new().route("/system/hardware", get(get_server_hardware))
}

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
pub(super) async fn get_server_hardware(
    State(state): State<AppState>,
) -> Result<Json<ServerHardwareResponse>, AppError> {
    Ok(Json(state.service.get_server_hardware().await?))
}
