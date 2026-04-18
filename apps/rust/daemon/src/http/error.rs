use axum::Json;
use axum::http::StatusCode;
use axum::http::header::{self, HeaderValue};
use axum::response::{IntoResponse, Response};
use synforge_core::{api::ApiError, error::SynforgeError};

#[derive(Debug)]
pub(crate) struct AppError {
    error: anyhow::Error,
    basic_challenge: bool,
}

impl AppError {
    pub(crate) fn auth(message: impl Into<String>) -> Self {
        Self {
            error: anyhow::anyhow!(SynforgeError::Unauthorized).context(message.into()),
            basic_challenge: false,
        }
    }

    pub(crate) fn unavailable(message: impl Into<String>) -> Self {
        Self {
            error: anyhow::anyhow!(message.into()),
            basic_challenge: false,
        }
    }

    pub(crate) fn with_basic_challenge(error: Self) -> Self {
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
