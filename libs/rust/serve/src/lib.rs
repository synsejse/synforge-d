mod api;

use std::sync::Arc;

use axum::extract::State;
use axum::http::header::{self, HeaderName, HeaderValue};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use synforge_core::ApiError;
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

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz() -> &'static str {
    "ready"
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
    let expected = state.service.config().bearer_token.clone();
    if expected.is_empty() {
        return Ok(next.run(request).await);
    }
    let value = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::auth("missing Authorization header"))?;
    let prefix = "Bearer ";
    if !value.starts_with(prefix) || value[prefix.len()..] != expected {
        return Err(AppError::auth("invalid bearer token"));
    }
    Ok(next.run(request).await)
}

#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl AppError {
    fn auth(message: impl Into<String>) -> Self {
        Self(anyhow::anyhow!(synforge_core::SynforgeError::Unauthorized).context(message.into()))
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
            if let Some(error) = cause.downcast_ref::<synforge_core::SynforgeError>() {
                match error {
                    synforge_core::SynforgeError::Unauthorized => {
                        status = StatusCode::UNAUTHORIZED;
                        code = "unauthorized";
                        message = error.to_string();
                    }
                    synforge_core::SynforgeError::NotFound(_) => {
                        status = StatusCode::NOT_FOUND;
                        code = "not_found";
                        message = error.to_string();
                    }
                    synforge_core::SynforgeError::Conflict(_) => {
                        status = StatusCode::CONFLICT;
                        code = "conflict";
                        message = error.to_string();
                    }
                    synforge_core::SynforgeError::BadRequest(_)
                    | synforge_core::SynforgeError::Spec(_)
                    | synforge_core::SynforgeError::Config(_) => {
                        status = StatusCode::BAD_REQUEST;
                        code = "bad_request";
                        message = error.to_string();
                    }
                    synforge_core::SynforgeError::Internal(_) => {
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
