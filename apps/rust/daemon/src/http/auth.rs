use axum::extract::State;
use axum::http::header;
use axum::http::{HeaderMap, Method};
use axum::middleware::Next;
use axum::response::Response;
use base64::Engine;
use synforge_core::constants::DEFAULT_SIGNING_PUBLIC_KEY_NAME;
use synforge_core::model::{UserAccount, UserPermission};

use super::session_cookie::{find_cookie, session_cookie_name};
use super::{AppError, AppState};

pub(crate) async fn authenticate_api_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: axum::extract::Request,
    next: Next,
) -> Result<Response, AppError> {
    if is_public_api_route(request.method(), request.uri().path()) {
        return Ok(next.run(request).await);
    }
    if !is_setup_complete(&state).await? {
        return Err(AppError::unavailable("daemon setup is not complete"));
    }
    let required = required_api_permission(request.method(), request.uri().path());
    let user = authenticate_session_headers(&state, &headers, required).await?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

pub(crate) async fn authenticate_repo_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: axum::extract::Request,
    next: Next,
) -> Result<Response, AppError> {
    if !is_setup_complete(&state).await? {
        return Err(AppError::unavailable("daemon setup is not complete"));
    }
    if is_public_repo_route(request.method(), request.uri().path()) {
        return Ok(next.run(request).await);
    }
    let user = authenticate_basic_headers(&state, &headers, UserPermission::Repo)
        .await
        .map_err(AppError::with_basic_challenge)?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

pub(crate) async fn authenticate_docs_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: Next,
) -> Result<Response, AppError> {
    if request.method() == Method::GET && request.uri().path() == "/docs" {
        return Ok(next.run(request).await);
    }
    if !is_setup_complete(&state).await? {
        return Err(AppError::unavailable("daemon setup is not complete"));
    }
    let _user = authenticate_session_headers(&state, &headers, UserPermission::Read).await?;
    Ok(next.run(request).await)
}

async fn authenticate_session_headers(
    state: &AppState,
    headers: &HeaderMap,
    required: UserPermission,
) -> Result<UserAccount, AppError> {
    let token = find_cookie(headers, session_cookie_name())
        .ok_or_else(|| AppError::auth("missing session cookie"))?;
    state
        .service
        .authorize_session(token, required)
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
    state
        .service
        .authenticate_user(&handle, &password, required)
        .await
        .map_err(AppError::from)
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
    // User management is privileged even for reads.
    if path.starts_with("/users") {
        return UserPermission::Write;
    }
    match (method, path) {
        // Privileged reads: these GETs expose sensitive material (full
        // effective config, exported signing key) and require Write.
        (&Method::GET, "/config/effective") | (&Method::GET, "/signing/export") => {
            UserPermission::Write
        }
        // Safe reads.
        (&Method::GET | &Method::HEAD, _) => UserPermission::Read,
        // Fail closed: every mutating method requires Write, including any
        // route not explicitly enumerated above.
        _ => UserPermission::Write,
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

async fn is_setup_complete(state: &AppState) -> Result<bool, AppError> {
    Ok(state
        .service
        .effective_config()
        .await?
        .config
        .bootstrap_completed)
}

fn is_public_repo_route(method: &Method, path: &str) -> bool {
    if *method != Method::GET {
        return false;
    }
    let normalized_path = path.trim_start_matches('/');
    normalized_path == DEFAULT_SIGNING_PUBLIC_KEY_NAME
        || normalized_path == format!("repo/{}", DEFAULT_SIGNING_PUBLIC_KEY_NAME)
}
