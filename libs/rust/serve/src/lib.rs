mod api;

use std::sync::Arc;

use axum::extract::State;
use axum::http::header::{self, HeaderName, HeaderValue};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use synforge_core::{api::ApiError, error::SynforgeError, validated::AuthToken};
use synforge_orchestrator::SynforgeService;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub service: Arc<SynforgeService>,
}

pub fn router(service: Arc<SynforgeService>) -> Router {
    let state = AppState {
        service: Arc::clone(&service),
    };
    let api = api::router(state.clone())
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            authenticate_request,
        ));

    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .nest("/api/v1", api)
        .nest_service("/repo", ServeDir::new(service.config().runtime_paths().repo_dir().to_path_buf()))
        .layer(middleware::map_response(add_security_headers))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn healthz(State(state): State<AppState>) -> Result<&'static str, AppError> {
    state.service.health_check().await?;
    Ok("ok")
}

async fn readyz(State(state): State<AppState>) -> Result<&'static str, AppError> {
    state.service.health_check().await?;
    Ok("ready")
}

async fn add_security_headers(mut response: Response) -> Response {
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

async fn authenticate_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: Next,
) -> Result<Response, AppError> {
    let Some(expected) = AuthToken::new(&state.service.config().bearer_token).ok() else {
        return Ok(next.run(request).await);
    };
    let value = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::auth("missing Authorization header"))?;
    let prefix = "Bearer ";
    if !value.starts_with(prefix) || !constant_time_eq(&value[prefix.len()..], expected.secret()) {
        return Err(AppError::auth("invalid bearer token"));
    }
    Ok(next.run(request).await)
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    let max_len = left.len().max(right.len());
    let mut diff = left.len() ^ right.len();

    for index in 0..max_len {
        let left_byte = left.get(index).copied().unwrap_or_default();
        let right_byte = right.get(index).copied().unwrap_or_default();
        diff |= usize::from(left_byte ^ right_byte);
    }

    diff == 0
}

#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl AppError {
    fn auth(message: impl Into<String>) -> Self {
        Self(anyhow::anyhow!(SynforgeError::Unauthorized).context(message.into()))
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let mut status = StatusCode::INTERNAL_SERVER_ERROR;
        let mut code = "internal_error";
        let mut message = self.0.to_string();
        for cause in self.0.chain() {
            if let Some(error) = cause.downcast_ref::<SynforgeError>() {
                match error {
                    SynforgeError::Unauthorized => {
                        status = StatusCode::UNAUTHORIZED;
                        code = "unauthorized";
                        message = error.to_string();
                    }
                    SynforgeError::NotFound(_) => {
                        status = StatusCode::NOT_FOUND;
                        code = "not_found";
                        message = error.to_string();
                    }
                    SynforgeError::Conflict(_) => {
                        status = StatusCode::CONFLICT;
                        code = "conflict";
                        message = error.to_string();
                    }
                    SynforgeError::BadRequest(_)
                    | SynforgeError::Spec(_)
                    | SynforgeError::Config(_) => {
                        status = StatusCode::BAD_REQUEST;
                        code = "bad_request";
                        message = error.to_string();
                    }
                    SynforgeError::Internal(_) => {
                        status = StatusCode::INTERNAL_SERVER_ERROR;
                        code = "internal_error";
                        message = error.to_string();
                    }
                }
                break;
            }
        }
        (status, Json(ApiError { code, message })).into_response()
    }
}
