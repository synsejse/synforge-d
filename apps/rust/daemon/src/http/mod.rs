mod artifacts;
mod auth;
mod cache;
mod config;
mod error;
mod hardware;
mod jobs;
mod logs;
mod openapi;
mod packages;
mod repo;
mod repo_files;
mod session;
mod session_cookie;
mod setup;
mod signing;
mod state;
mod sync;
mod system_routes;
mod users;

use std::path::PathBuf;
use std::sync::Arc;

use crate::SynforgeService;
use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::middleware;
use axum::routing::get;
use axum_helmet::{Helmet, HelmetLayer};
use helmet_core::{CrossOriginOpenerPolicy, ReferrerPolicy, XContentTypeOptions, XFrameOptions};
use synforge_core::constants::DEFAULT_WEBUI_MAX_REQUEST_BODY_BYTES;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use self::auth::{authenticate_api_request, authenticate_docs_request, authenticate_repo_request};
use self::repo_files::{download_repo_file, repo_root};
use self::system_routes::{healthz, readyz};

pub(crate) use self::error::AppError;
pub use self::openapi::ApiDoc;
pub(crate) use self::session_cookie::{clear_session_cookie, create_session_cookie};
pub(crate) use self::state::AppState;

pub fn router(service: Arc<SynforgeService>, web_root: PathBuf) -> Router {
    let state = AppState {
        service: Arc::clone(&service),
    };
    let security_layer: HelmetLayer = Helmet::new()
        .add(XContentTypeOptions::nosniff())
        .add(XFrameOptions::deny())
        .add(ReferrerPolicy::no_referrer())
        .add(CrossOriginOpenerPolicy::same_origin())
        .try_into()
        .expect("failed to construct HelmetLayer");
    let api = packages::router()
        .merge(jobs::router())
        .merge(artifacts::router())
        .merge(logs::router())
        .merge(sync::router())
        .merge(repo::router())
        .merge(cache::router())
        .merge(hardware::router())
        .merge(signing::router())
        .merge(users::router())
        .merge(session::router())
        .merge(setup::router())
        .merge(config::router())
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            authenticate_api_request,
        ));
    let docs = Router::new()
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
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

    Router::<AppState>::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .merge(docs)
        .nest("/api/v1", api)
        .nest("/repo", repo)
        .fallback_service(
            ServeDir::new(&web_root).not_found_service(ServeFile::new(web_root.join("index.html"))),
        )
        .layer(DefaultBodyLimit::max(DEFAULT_WEBUI_MAX_REQUEST_BODY_BYTES))
        .layer(security_layer)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
