use axum::http::HeaderValue;
use base64::Engine;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::AppError;

type HmacSha256 = Hmac<Sha256>;
const SESSION_COOKIE_NAME: &str = "synforge_session";
const SESSION_TTL_HOURS: i64 = 24 * 7;

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
    let mut cookie = format!("{SESSION_COOKIE_NAME}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax");
    if secure {
        cookie.push_str("; Secure");
    }
    HeaderValue::from_str(&cookie).map_err(AppError::from)
}

pub(crate) fn decode_session_cookie(
    cookie: &str,
    secret: &[u8],
) -> Result<UiSessionClaims, AppError> {
    let (payload, signature) = cookie
        .split_once('.')
        .ok_or_else(|| AppError::auth("invalid session cookie"))?;
    let mut mac =
        HmacSha256::new_from_slice(secret).map_err(|_| AppError::auth("invalid session secret"))?;
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

fn encode_session_cookie(claims: &UiSessionClaims, secret: &[u8]) -> Result<String, AppError> {
    let payload = serde_json::to_vec(claims).map_err(AppError::from)?;
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload);
    let mut mac =
        HmacSha256::new_from_slice(secret).map_err(|_| AppError::auth("invalid session secret"))?;
    mac.update(payload.as_bytes());
    let signature =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    Ok(format!("{payload}.{signature}"))
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
