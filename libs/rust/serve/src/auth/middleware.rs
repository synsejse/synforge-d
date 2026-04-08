use axum::extract::State;
use axum::http::header;
use axum::http::{HeaderMap, Method};
use axum::middleware::Next;
use axum::response::Response;
use base64::Engine;
use synforge_core::constants::DEFAULT_SIGNING_PUBLIC_KEY_NAME;
use synforge_core::model::{UserAccount, UserPermission};

use super::session::{decode_session_cookie, find_cookie, session_cookie_name};
use crate::{AppError, AppState};

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
    let cookie_value = find_cookie(headers, session_cookie_name())
        .ok_or_else(|| AppError::auth("missing session cookie"))?;
    let claims = decode_session_cookie(
        cookie_value,
        state.service.config().session_secret.as_bytes(),
    )?;
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
    if path.starts_with("/users") {
        return UserPermission::Write;
    }
    match (method, path) {
        (&Method::GET, "/config/effective") | (&Method::GET, "/signing/export") => {
            UserPermission::Write
        }
        (&Method::POST, "/packages")
        | (&Method::POST, "/packages/refresh-all")
        | (&Method::POST, "/repositories/browse")
        | (&Method::POST, "/jobs/prune-failed")
        | (&Method::POST, "/config/runtime") => UserPermission::Write,
        (&Method::POST, path) if path.starts_with("/jobs/") && path.ends_with("/kill") => {
            UserPermission::Write
        }
        (&Method::POST, path) if path.starts_with("/jobs/") && path.ends_with("/retry") => {
            UserPermission::Write
        }
        (&Method::POST, path) if path.starts_with("/signing/") => UserPermission::Write,
        (&Method::POST, path) if path.ends_with("/rebuild") || path.ends_with("/refresh") => {
            UserPermission::Write
        }
        (&Method::PUT, path) if path.starts_with("/packages/") => UserPermission::Write,
        (&Method::PUT, path) if path.starts_with("/users/") => UserPermission::Write,
        (&Method::DELETE, path)
            if path.starts_with("/packages/")
                || path.starts_with("/jobs/")
                || path.starts_with("/users/")
                || path.starts_with("/signing/") =>
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
