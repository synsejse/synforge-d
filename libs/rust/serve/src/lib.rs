mod api;
mod auth;
pub mod openapi;
mod repo_files;
mod system_routes;

use std::sync::Arc;

use crate::auth::middleware::{
    authenticate_api_request, authenticate_docs_request, authenticate_repo_request,
};
pub(crate) use crate::auth::session::{clear_session_cookie, create_session_cookie};
use crate::repo_files::{download_repo_file, repo_root};
use crate::system_routes::{add_security_headers, healthz, readyz};
use axum::http::StatusCode;
use axum::http::header::{self, HeaderValue};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use synforge_core::{api::ApiError, error::SynforgeError};
use synforge_orchestrator::SynforgeService;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub service: Arc<SynforgeService>,
}

pub fn router(service: Arc<SynforgeService>) -> Router {
    let state = AppState {
        service: Arc::clone(&service),
    };
    let api =
        api::router(state.clone())
            .with_state(state.clone())
            .layer(middleware::from_fn_with_state(
                state.clone(),
                authenticate_api_request,
            ));
    let docs = Router::new()
        .merge(SwaggerUi::new("/docs").url("/openapi.json", openapi::ApiDoc::openapi()))
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            authenticate_docs_request,
        ));
    let repo = Router::new()
        .route("/", get(repo_root))
        .route("/{*path}", get(download_repo_file))
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            authenticate_repo_request,
        ));

    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .merge(docs)
        .nest("/api/v1", api)
        .nest("/repo", repo)
        .layer(middleware::map_response(add_security_headers))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[derive(Debug)]
pub struct AppError {
    error: anyhow::Error,
    basic_challenge: bool,
}

impl AppError {
    fn auth(message: impl Into<String>) -> Self {
        Self {
            error: anyhow::anyhow!(SynforgeError::Unauthorized).context(message.into()),
            basic_challenge: false,
        }
    }

    fn unavailable(message: impl Into<String>) -> Self {
        Self {
            error: anyhow::anyhow!(message.into()),
            basic_challenge: false,
        }
    }

    fn with_basic_challenge(error: Self) -> Self {
        Self {
            error: error.error,
            basic_challenge: true,
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self {
            error: value.into(),
            basic_challenge: false,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let mut status = StatusCode::INTERNAL_SERVER_ERROR;
        let mut code = "internal_error";
        let mut message = self.error.to_string();
        for cause in self.error.chain() {
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
        if message == "daemon setup is not complete" {
            status = StatusCode::SERVICE_UNAVAILABLE;
        }
        let mut response = (status, Json(ApiError { code, message })).into_response();
        if status == StatusCode::UNAUTHORIZED && self.basic_challenge {
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static("Basic realm=\"Synforge\""),
            );
        }
        response
    }
}
