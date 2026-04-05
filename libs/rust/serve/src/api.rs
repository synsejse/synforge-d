use axum::Router;
use axum::routing::{get, post};

use crate::AppState;

pub(crate) mod config;
pub(crate) mod jobs;
pub(crate) mod logs;
pub(crate) mod packages;
pub(crate) mod repo;
pub(crate) mod session;
pub(crate) mod users;
pub(crate) use config::{get_config_schema, get_effective_config, update_runtime_settings};
pub(crate) use jobs::{
    delete_job, download_job_artifact, get_job, get_job_artifact_meta, list_active_jobs,
    list_completed_jobs, list_job_artifacts, list_jobs, prune_failed_jobs,
};
pub(crate) use logs::{
    get_job_log_chunk_by_source, get_job_log_manifest, get_job_log_meta_by_source,
};
pub(crate) use packages::{
    create_package, delete_package, get_package, get_package_builds, list_mock_chroots,
    list_packages, trigger_rebuild, trigger_refresh, trigger_target_rebuild,
    trigger_target_refresh, update_package,
};
pub(crate) use repo::{browse_repository, get_repo_inventory, get_repo_summary};
pub(crate) use session::{
    get_session, get_setup_status, initialize_setup, login_session, logout_session,
};
pub(crate) use users::{
    change_user_password, create_user, delete_user, get_user_metrics, list_users, update_user,
};

pub fn router(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/packages", get(list_packages).post(create_package))
        .route("/mock/chroots", get(list_mock_chroots))
        .route("/repositories/browse", post(browse_repository))
        .route(
            "/packages/{name}",
            get(get_package).put(update_package).delete(delete_package),
        )
        .route("/packages/{name}/builds", get(get_package_builds))
        .route("/packages/{name}/rebuild", post(trigger_rebuild))
        .route("/packages/{name}/refresh", post(trigger_refresh))
        .route(
            "/packages/{name}/targets/{mock_chroot}/rebuild",
            post(trigger_target_rebuild),
        )
        .route(
            "/packages/{name}/targets/{mock_chroot}/refresh",
            post(trigger_target_refresh),
        )
        .route("/jobs", get(list_jobs))
        .route("/jobs/active", get(list_active_jobs))
        .route("/jobs/completed", get(list_completed_jobs))
        .route("/jobs/prune-failed", post(prune_failed_jobs))
        .route("/session", get(get_session))
        .route("/session/login", post(login_session))
        .route("/session/logout", post(logout_session))
        .route("/setup/status", get(get_setup_status))
        .route("/setup/initialize", post(initialize_setup))
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/{id}",
            get(get_user_metrics).put(update_user).delete(delete_user),
        )
        .route("/users/{id}/password", post(change_user_password))
        .route("/jobs/{id}", get(get_job).delete(delete_job))
        .route("/jobs/{id}/artifacts", get(list_job_artifacts))
        .route(
            "/jobs/{id}/artifacts/{file}/meta",
            get(get_job_artifact_meta),
        )
        .route("/jobs/{id}/logs", get(get_job_log_manifest))
        .route(
            "/jobs/{id}/logs/{source}/meta",
            get(get_job_log_meta_by_source),
        )
        .route(
            "/jobs/{id}/logs/{source}/stream",
            get(get_job_log_chunk_by_source),
        )
        .route("/jobs/{id}/artifacts/{file}", get(download_job_artifact))
        .route("/repo/files", get(get_repo_inventory))
        .route("/repo/summary", get(get_repo_summary))
        .route("/config/schema", get(get_config_schema))
        .route("/config/effective", get(get_effective_config))
        .route("/config/runtime", post(update_runtime_settings))
}
