mod api;

use std::sync::Arc;
use std::path::PathBuf;

use axum::extract::State;
use axum::body::Body;
use axum::extract::{Extension, Path};
use axum::http::header::{self, HeaderName, HeaderValue};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use synforge_core::{
    api::ApiError,
    error::SynforgeError,
    model::{UserAccount, UserPermission},
};
use synforge_orchestrator::SynforgeService;
use time::{Duration, OffsetDateTime};
use tower_http::trace::TraceLayer;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;
const SESSION_COOKIE_NAME: &str = "synforge_session";
const SESSION_TTL_HOURS: i64 = 24 * 7;

#[derive(Clone)]
pub struct AppState {
    pub service: Arc<SynforgeService>,
    pub config_path: PathBuf,
}

pub fn router(service: Arc<SynforgeService>) -> Router {
    let config_path = synforge_core::config::DaemonConfig::config_path();
    let state = AppState {
        service: Arc::clone(&service),
        config_path,
    };
    let api = api::router(state.clone())
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            authenticate_api_request,
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
        .nest("/api/v1", api)
        .nest("/repo", repo)
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

async fn authenticate_api_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: axum::extract::Request,
    next: Next,
) -> Result<Response, AppError> {
    if is_public_api_route(request.method(), request.uri().path()) {
        return Ok(next.run(request).await);
    }
    if !is_setup_complete(&state)? {
        return Err(AppError::unavailable("daemon setup is not complete"));
    }
    let required = required_api_permission(request.method(), request.uri().path());
    let user = authenticate_session_headers(&state, &headers, required).await?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

async fn authenticate_repo_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: axum::extract::Request,
    next: Next,
) -> Result<Response, AppError> {
    if !is_setup_complete(&state)? {
        return Err(AppError::unavailable("daemon setup is not complete"));
    }
    let user = authenticate_basic_headers(&state, &headers, UserPermission::Repo)
        .await
        .map_err(AppError::with_basic_challenge)?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

async fn authenticate_session_headers(
    state: &AppState,
    headers: &HeaderMap,
    required: UserPermission,
) -> Result<UserAccount, AppError> {
    let cookie_value = find_cookie(headers, SESSION_COOKIE_NAME)
        .ok_or_else(|| AppError::auth("missing session cookie"))?;
    let claims = decode_session_cookie(cookie_value, state.service.config().session_secret.as_bytes())?;
    state
        .service
        .authorize_user(claims.user_id, required)
        .await
        .map_err(AppError::from)
}

async fn authenticate_basic_headers(
    state: &AppState,
    headers: &HeaderMap,
    required: UserPermission,
) -> Result<UserAccount, AppError> {
    let encoded = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(parse_basic_authorization)
        .ok_or_else(|| AppError::auth("invalid credentials"))?;
    let (handle, password) = decode_basic_credentials(encoded)?;
    Ok(state
        .service
        .authenticate_user(&handle, &password, required)
        .await
        .map_err(AppError::from)?)
}

fn parse_basic_authorization(value: &str) -> Option<&str> {
    value.strip_prefix("Basic ")
}

fn decode_basic_credentials(encoded: &str) -> Result<(String, String), AppError> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| AppError::auth("invalid credentials"))?;
    let decoded = String::from_utf8(decoded).map_err(|_| AppError::auth("invalid credentials"))?;
    let (handle, password) = decoded
        .split_once(':')
        .ok_or_else(|| AppError::auth("invalid credentials"))?;
    Ok((handle.to_string(), password.to_string()))
}

fn required_api_permission(method: &Method, path: &str) -> UserPermission {
    let path = path.strip_prefix("/api/v1").unwrap_or(path);
    if path.starts_with("/users") {
        return UserPermission::Write;
    }
    match (method, path) {
        (&Method::POST, "/packages")
        | (&Method::POST, "/repositories/browse")
        | (&Method::POST, "/jobs/prune-failed")
        | (&Method::POST, "/config/runtime") => UserPermission::Write,
        (&Method::POST, path) if path.ends_with("/rebuild") || path.ends_with("/refresh") => {
            UserPermission::Write
        }
        (&Method::PUT, path) if path.starts_with("/packages/") => UserPermission::Write,
        (&Method::PUT, path) if path.starts_with("/users/") => UserPermission::Write,
        (&Method::DELETE, path)
            if path.starts_with("/packages/")
                || path.starts_with("/jobs/")
                || path.starts_with("/users/") =>
        {
            UserPermission::Write
        }
        _ => UserPermission::Read,
    }
}

fn is_public_api_route(method: &Method, path: &str) -> bool {
    let path = path.strip_prefix("/api/v1").unwrap_or(path);
    matches!(
        (method, path),
        (&Method::GET, "/setup/status")
            | (&Method::GET, "/config/schema")
            | (&Method::POST, "/setup/initialize")
            | (&Method::POST, "/session/login")
            | (&Method::POST, "/session/logout")
    )
}

fn is_setup_complete(state: &AppState) -> Result<bool, AppError> {
    Ok(
        synforge_core::config::DaemonConfig::load_from_file(&state.config_path)
            .map(|config| config.bootstrap_completed)
            .unwrap_or(false),
    )
}

fn find_cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    let header = headers.get(header::COOKIE)?.to_str().ok()?;
    header.split(';').map(str::trim).find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name).then_some(value)
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UiSessionClaims {
    pub user_id: Uuid,
    pub issued_at: i64,
    pub expires_at: i64,
}

pub(crate) fn create_session_cookie(
    user_id: Uuid,
    secret: &[u8],
    secure: bool,
) -> Result<HeaderValue, AppError> {
    let now = OffsetDateTime::now_utc();
    let claims = UiSessionClaims {
        user_id,
        issued_at: now.unix_timestamp(),
        expires_at: (now + Duration::hours(SESSION_TTL_HOURS)).unix_timestamp(),
    };
    let encoded = encode_session_cookie(&claims, secret)?;
    let mut cookie = format!(
        "{SESSION_COOKIE_NAME}={encoded}; Path=/; Max-Age={}; HttpOnly; SameSite=Lax",
        Duration::hours(SESSION_TTL_HOURS).whole_seconds()
    );
    if secure {
        cookie.push_str("; Secure");
    }
    HeaderValue::from_str(&cookie).map_err(AppError::from)
}

pub(crate) fn clear_session_cookie(secure: bool) -> Result<HeaderValue, AppError> {
    let mut cookie = format!(
        "{SESSION_COOKIE_NAME}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax"
    );
    if secure {
        cookie.push_str("; Secure");
    }
    HeaderValue::from_str(&cookie).map_err(AppError::from)
}

fn encode_session_cookie(claims: &UiSessionClaims, secret: &[u8]) -> Result<String, AppError> {
    let payload = serde_json::to_vec(claims).map_err(AppError::from)?;
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload);
    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|_| AppError::auth("invalid session secret"))?;
    mac.update(payload.as_bytes());
    let signature = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    Ok(format!("{payload}.{signature}"))
}

fn decode_session_cookie(cookie: &str, secret: &[u8]) -> Result<UiSessionClaims, AppError> {
    let (payload, signature) = cookie
        .split_once('.')
        .ok_or_else(|| AppError::auth("invalid session cookie"))?;
    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|_| AppError::auth("invalid session secret"))?;
    mac.update(payload.as_bytes());
    let signature = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|_| AppError::auth("invalid session cookie"))?;
    mac.verify_slice(&signature)
        .map_err(|_| AppError::auth("invalid session cookie"))?;
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| AppError::auth("invalid session cookie"))?;
    let claims = serde_json::from_slice::<UiSessionClaims>(&payload)
        .map_err(|_| AppError::auth("invalid session cookie"))?;
    if claims.expires_at <= OffsetDateTime::now_utc().unix_timestamp() {
        return Err(AppError::auth("session expired"));
    }
    Ok(claims)
}

async fn repo_root() -> Result<StatusCode, AppError> {
    Err(AppError::from(anyhow::anyhow!(SynforgeError::NotFound(
        "repository root".to_string()
    ))))
}

async fn download_repo_file(
    Extension(user): Extension<UserAccount>,
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let resolved = state.service.resolve_repo_file_path(&path).await?;
    let metadata = tokio::fs::metadata(&resolved).await?;
    let file = tokio::fs::File::open(&resolved).await?;
    state
        .service
        .increment_user_download_bytes(user.id, metadata.len())
        .await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    Ok((headers, Body::from_stream(ReaderStream::new(file))))
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
