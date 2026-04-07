use std::path::PathBuf;

use anyhow::Context;
use axum::body::{Body, to_bytes};
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::header::{self, HeaderName};
use axum::http::{HeaderMap, HeaderValue, Response, StatusCode};
use axum::response::Redirect;
use axum::routing::any;
use axum::{Router, routing::get};
use axum_helmet::{Helmet, HelmetLayer};
use helmet_core::{CrossOriginOpenerPolicy, ReferrerPolicy, XContentTypeOptions, XFrameOptions};
use reqwest::Client;
use synforge_core::constants::{
    DEFAULT_DAEMON_HTTP_PORT, DEFAULT_WEBUI_LISTEN_ADDR, DEFAULT_WEBUI_MAX_REQUEST_BODY_BYTES,
    DEFAULT_WEBUI_STATIC_DIR,
};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    daemon_base_url: String,
    client: Client,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    synforge_core::logging::init_tracing();

    let listen_addr = DEFAULT_WEBUI_LISTEN_ADDR.to_string();
    let daemon_base_url =
        normalize_base_url(&format!("http://daemon:{}", DEFAULT_DAEMON_HTTP_PORT));
    let static_dir = PathBuf::from(DEFAULT_WEBUI_STATIC_DIR);
    let index_path = static_dir.join("index.html");

    let state = AppState {
        daemon_base_url,
        client: Client::builder().build()?,
    };
    let security_layer: HelmetLayer = Helmet::new()
        .add(XContentTypeOptions::nosniff())
        .add(XFrameOptions::deny())
        .add(ReferrerPolicy::no_referrer())
        .add(CrossOriginOpenerPolicy::same_origin())
        .try_into()
        .expect("failed to construct HelmetLayer");

    let app = Router::new()
        .route("/healthz", get(proxy_to_daemon))
        .route("/readyz", get(proxy_to_daemon))
        .route("/api/{*path}", any(proxy_to_daemon))
        .route("/docs", get(swagger_docs_redirect))
        .route("/docs/", any(proxy_to_daemon))
        .route("/docs/{*path}", any(proxy_to_daemon))
        .route("/openapi.json", any(proxy_to_daemon))
        .route("/swagger-ui.css", any(proxy_to_daemon))
        .route("/swagger-ui-bundle.js", any(proxy_to_daemon))
        .route("/swagger-ui-standalone-preset.js", any(proxy_to_daemon))
        .route("/swagger-initializer.js", any(proxy_to_daemon))
        .route("/index.css", any(proxy_to_daemon))
        .route("/oauth2-redirect.html", any(proxy_to_daemon))
        .route("/repo", any(proxy_to_daemon))
        .route("/repo/{*path}", any(proxy_to_daemon))
        .fallback_service(ServeDir::new(static_dir).not_found_service(ServeFile::new(index_path)))
        .layer(security_layer)
        .layer(DefaultBodyLimit::max(DEFAULT_WEBUI_MAX_REQUEST_BODY_BYTES))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr)
        .await
        .with_context(|| format!("failed to bind {}", listen_addr))?;
    let bound = listener
        .local_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|_| listen_addr.clone());
    tracing::info!("webui listening on {}", bound);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler");
        signal.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    tracing::info!("webui shutdown signal received");
}

async fn swagger_docs_redirect() -> Redirect {
    Redirect::permanent("/docs/")
}

fn normalize_base_url(value: &str) -> String {
    value.trim_end_matches('/').to_string()
}

async fn proxy_to_daemon(
    State(state): State<AppState>,
    request: Request,
) -> Result<Response<Body>, (StatusCode, String)> {
    let (parts, body) = request.into_parts();
    let method = parts.method;
    let uri = parts.uri;
    let headers = parts.headers;
    let target_url = format!("{}{}", state.daemon_base_url, uri);
    let mut builder = state.client.request(method, target_url);

    for (name, value) in forward_request_headers(&headers) {
        builder = builder.header(name, value);
    }

    let body = to_bytes(body, DEFAULT_WEBUI_MAX_REQUEST_BODY_BYTES)
        .await
        .map_err(|error| (StatusCode::PAYLOAD_TOO_LARGE, error.to_string()))?;
    if !body.is_empty() {
        builder = builder.body(body.clone());
    }

    let upstream = builder
        .send()
        .await
        .map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))?;

    let status = upstream.status();
    let headers = upstream.headers().clone();
    let body = upstream
        .bytes()
        .await
        .map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))?;

    let mut response = Response::builder().status(status);
    for (name, value) in forward_response_headers(&headers) {
        response = response.header(name, value);
    }

    response
        .body(Body::from(body))
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
}

fn forward_request_headers(headers: &HeaderMap) -> Vec<(HeaderName, HeaderValue)> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            if is_hop_by_hop_header(name) || *name == header::HOST {
                None
            } else {
                Some((name.clone(), value.clone()))
            }
        })
        .collect()
}

fn forward_response_headers(headers: &HeaderMap) -> Vec<(HeaderName, HeaderValue)> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            if is_hop_by_hop_header(name) {
                None
            } else {
                Some((name.clone(), value.clone()))
            }
        })
        .collect()
}

fn is_hop_by_hop_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
    )
}
