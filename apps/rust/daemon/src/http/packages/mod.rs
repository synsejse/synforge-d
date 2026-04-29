mod actions;
mod crud;
mod lookups;
mod refresh_all;

pub(super) use actions::{
    __path_trigger_rebuild, __path_trigger_refresh, __path_trigger_target_rebuild,
    __path_trigger_target_refresh, trigger_rebuild, trigger_refresh, trigger_target_rebuild,
    trigger_target_refresh,
};
pub(super) use crud::{
    __path_create_package, __path_delete_package, __path_get_package, __path_list_packages,
    __path_update_package, create_package, delete_package, get_package, list_packages,
    update_package,
};
pub(super) use lookups::{
    __path_browse_repository, __path_get_package_builds, __path_list_mock_chroots,
    browse_repository, get_package_builds, list_mock_chroots,
};
pub(super) use refresh_all::{
    __path_get_refresh_all_packages_progress, __path_trigger_refresh_all_packages,
    get_refresh_all_packages_progress, trigger_refresh_all_packages,
};

use super::AppState;
use axum::Router;
use axum::routing::{get, post};

pub fn router() -> Router<AppState> {
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
}
