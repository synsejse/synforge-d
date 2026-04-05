use axum::Json;
use axum::extract::{Extension, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use synforge_core::api::{SessionLoginRequest, SessionResponse};
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
