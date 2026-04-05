use axum::extract::State;
use axum::http::header::{HeaderName, HeaderValue};
use axum::response::Response;

use crate::{AppError, AppState};

pub(crate) async fn healthz(State(state): State<AppState>) -> Result<&'static str, AppError> {
    state.service.health_check().await?;
    Ok("ok")
}

pub(crate) async fn readyz(State(state): State<AppState>) -> Result<&'static str, AppError> {
    state.service.health_check().await?;
    Ok("ready")
}

pub(crate) async fn add_security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static("same-origin"),
    );
    response
}
