use axum::Router;
use axum::routing::{get, post};

use crate::AppState;

pub(crate) mod artifacts;
pub(crate) mod cache;
pub(crate) mod config;
pub(crate) mod hardware;
pub(crate) mod jobs;
pub(crate) mod logs;
pub(crate) mod packages;
pub(crate) mod repo;
pub(crate) mod session;
pub(crate) mod setup;
pub(crate) mod signing;
pub(crate) mod sync;
pub(crate) mod users;
pub(crate) use artifacts::{download_job_artifact, get_job_artifact_meta, list_job_artifacts};
pub(crate) use cache::get_cache_stats;
pub(crate) use config::{get_config_schema, get_effective_config, update_runtime_settings};
pub(crate) use hardware::get_server_hardware;
pub(crate) use jobs::{
    delete_job, get_job, get_job_usage, kill_job, list_job_usage, list_jobs, prune_failed_jobs,
    retry_job,
};
pub(crate) use logs::{
    get_job_log_chunk_by_source, get_job_log_manifest, get_job_log_meta_by_source,
};
pub(crate) use packages::{
    create_package, delete_package, get_package, get_package_builds,
    get_refresh_all_packages_progress, list_mock_chroots, list_packages, trigger_rebuild,
    trigger_refresh, trigger_refresh_all_packages, trigger_target_rebuild, trigger_target_refresh,
    update_package,
};
pub(crate) use repo::{
    browse_repository, get_browse_repository_progress, get_repo_inventory, get_repo_summary,
};
pub(crate) use session::{get_session, login_session, logout_session};
pub(crate) use setup::{get_setup_status, initialize_setup};
pub(crate) use signing::{
    export_repo_signing_key, export_repo_signing_public_key, generate_repo_signing_key,
    get_repo_signing_reconcile_progress, get_repo_signing_status, import_repo_signing_key,
    remove_repo_signing_key, test_repo_signing, update_repo_signing_config,
};
pub(crate) use sync::{get_sync_metrics, list_package_sync_operations, list_sync_operations};
pub(crate) use users::{
    change_user_password, create_user, delete_user, get_user_metrics, list_users, update_user,
};

pub fn router(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/packages", get(list_packages).post(create_package))
        .route("/mock/chroots", get(list_mock_chroots))
        .route("/repositories/browse", post(browse_repository))
        .route(
            "/repositories/browse/progress",
            get(get_browse_repository_progress),
        )
        .route(
            "/packages/{name}",
            get(get_package).put(update_package).delete(delete_package),
        )
        .route("/packages/{name}/builds", get(get_package_builds))
        .route("/packages/{name}/rebuild", post(trigger_rebuild))
        .route("/packages/{name}/refresh", post(trigger_refresh))
        .route("/packages/refresh-all", post(trigger_refresh_all_packages))
        .route(
            "/packages/refresh-all/progress",
            get(get_refresh_all_packages_progress),
        )
        .route(
            "/packages/{name}/targets/{mock_chroot}/rebuild",
            post(trigger_target_rebuild),
        )
        .route(
            "/packages/{name}/targets/{mock_chroot}/refresh",
            post(trigger_target_refresh),
        )
        .route(
            "/packages/{name}/sync/operations",
            get(list_package_sync_operations),
        )
        .route("/jobs", get(list_jobs))
        .route("/jobs/usage", get(list_job_usage))
        .route("/jobs/prune-failed", post(prune_failed_jobs))
        .route("/jobs/{id}/kill", post(kill_job))
        .route("/jobs/{id}/retry", post(retry_job))
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
        .route("/jobs/{id}/usage", get(get_job_usage))
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
            "/jobs/{id}/logs/{source}/chunks",
            get(get_job_log_chunk_by_source),
        )
        .route(
            "/jobs/{id}/artifacts/{file}/content",
            get(download_job_artifact),
        )
        .route("/repo/files", get(get_repo_inventory))
        .route("/repo/summary", get(get_repo_summary))
        .route("/signing/status", get(get_repo_signing_status))
        .route(
            "/signing/reconcile/progress",
            get(get_repo_signing_reconcile_progress),
        )
        .route("/signing/export", get(export_repo_signing_key))
        .route(
            "/signing/export/public",
            get(export_repo_signing_public_key),
        )
        .route("/signing/generate", post(generate_repo_signing_key))
        .route("/signing/config", post(update_repo_signing_config))
        .route("/signing/import", post(import_repo_signing_key))
        .route(
            "/signing/key",
            axum::routing::delete(remove_repo_signing_key),
        )
        .route("/signing/test", post(test_repo_signing))
        .route("/sync/operations", get(list_sync_operations))
        .route("/sync/metrics", get(get_sync_metrics))
        .route("/cache/stats", get(get_cache_stats))
        .route("/system/hardware", get(get_server_hardware))
        .route("/config/schema", get(get_config_schema))
        .route("/config/effective", get(get_effective_config))
        .route("/config/runtime", post(update_runtime_settings))
}
