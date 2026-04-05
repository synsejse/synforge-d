use axum::Json;
use axum::extract::State;
use synforge_core::api::{ConfigSchemaResponse, EffectiveConfigDto, UpdateRuntimeSettingsRequest};

use crate::{AppError, AppState};

#[utoipa::path(
    get,
    path = "/api/v1/config/effective",
    tag = "Settings",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get effective daemon configuration", body = EffectiveConfigDto),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_effective_config(
    State(state): State<AppState>,
) -> Result<Json<EffectiveConfigDto>, AppError> {
    Ok(Json(state.service.effective_config().await))
}

#[utoipa::path(
    get,
    path = "/api/v1/config/schema",
    tag = "Settings",
    responses(
        (status = 200, description = "Get editable daemon configuration schema", body = ConfigSchemaResponse)
    )
)]
pub(crate) async fn get_config_schema(
    State(state): State<AppState>,
) -> Result<Json<ConfigSchemaResponse>, AppError> {
    Ok(Json(state.service.config_schema().await))
}

#[utoipa::path(
    post,
    path = "/api/v1/config/runtime",
    tag = "Settings",
    request_body = UpdateRuntimeSettingsRequest,
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Update runtime-editable daemon configuration", body = EffectiveConfigDto),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn update_runtime_settings(
    State(state): State<AppState>,
    Json(request): Json<UpdateRuntimeSettingsRequest>,
) -> Result<Json<EffectiveConfigDto>, AppError> {
    Ok(Json(state.service.update_runtime_settings(request).await?))
}
