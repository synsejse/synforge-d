use axum::http::HeaderValue;
use synforge_state::UI_SESSION_TTL_SECONDS;

use super::AppError;

const SESSION_COOKIE_NAME: &str = "synforge_session";

pub(crate) fn create_session_cookie(token: &str, secure: bool) -> Result<HeaderValue, AppError> {
    let mut cookie = format!(
        "{SESSION_COOKIE_NAME}={token}; Path=/; Max-Age={UI_SESSION_TTL_SECONDS}; HttpOnly; SameSite=Lax"
    );
    if secure {
        cookie.push_str("; Secure");
    }
    HeaderValue::from_str(&cookie).map_err(AppError::from)
}

pub(crate) fn clear_session_cookie(secure: bool) -> Result<HeaderValue, AppError> {
    let mut cookie = format!("{SESSION_COOKIE_NAME}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax");
    if secure {
        cookie.push_str("; Secure");
    }
    HeaderValue::from_str(&cookie).map_err(AppError::from)
}

pub(crate) fn find_cookie<'a>(headers: &'a axum::http::HeaderMap, name: &str) -> Option<&'a str> {
    let header = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    header.split(';').map(str::trim).find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name).then_some(value)
    })
}

pub(crate) fn session_cookie_name() -> &'static str {
    SESSION_COOKIE_NAME
}
