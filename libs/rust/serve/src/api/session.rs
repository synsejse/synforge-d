use axum::Json;
use axum::extract::{Extension, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use synforge_core::api::{
    EffectiveConfigDto, SessionLoginRequest, SessionResponse, SetupInitializeRequest,
    SetupStatusResponse,
};
use synforge_core::model::{UserAccount, UserPermission};

use crate::{AppError, AppState, clear_session_cookie, create_session_cookie};

#[utoipa::path(
    get,
    path = "/api/v1/session",
    tag = "Session",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get current UI session", body = SessionResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn get_session(
    Extension(user): Extension<UserAccount>,
    State(state): State<AppState>,
) -> Result<Json<SessionResponse>, AppError> {
    Ok(Json(state.service.get_session(user).await))
}

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

#[utoipa::path(
    post,
    path = "/api/v1/session/login",
    tag = "Session",
    request_body = SessionLoginRequest,
    responses(
        (status = 200, description = "Authenticate and create a UI session", body = SessionResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 503, body = synforge_core::api::ApiError)
    )
)]
pub(crate) async fn login_session(
    State(state): State<AppState>,
    Json(request): Json<SessionLoginRequest>,
) -> Result<Response, AppError> {
    let user = state
        .service
        .authenticate_user(&request.handle, &request.password, UserPermission::Read)
        .await?;
    let mut response = Json(state.service.get_session(user.clone()).await).into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        create_session_cookie(
            user.id,
            state.service.config().session_secret.as_bytes(),
            state
                .service
                .config()
                .public_base_url
                .starts_with("https://"),
        )?,
    );
    Ok(response)
}

#[utoipa::path(
    post,
    path = "/api/v1/session/logout",
    tag = "Session",
    responses(
        (status = 204, description = "Clear current UI session")
    )
)]
pub(crate) async fn logout_session(State(state): State<AppState>) -> Result<Response, AppError> {
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        clear_session_cookie(
            state
                .service
                .config()
                .public_base_url
                .starts_with("https://"),
        )?,
    );
    Ok(response)
}
