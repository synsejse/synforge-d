use super::{AppError, AppState};
use axum::Json;
use axum::Router;
use axum::extract::{Extension, State};
use axum::routing::{get, post};
use synforge_core::api::{
    ExportRepoSigningKeyResponse, ExportRepoSigningPublicKeyResponse,
    GenerateRepoSigningKeyResponse, ImportRepoSigningKeyRequest, ImportRepoSigningKeyResponse,
    RepoSigningReconcileProgressResponse, RepoSigningStatusResponse, TestRepoSigningResponse,
    UpdateRepoSigningConfigRequest,
};
use synforge_core::model::UserAccount;

pub fn router() -> Router<AppState> {
    Router::new()
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
}

#[utoipa::path(
    get,
    path = "/api/v1/signing/status",
    tag = "Signing",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get repository signing status", body = RepoSigningStatusResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn get_repo_signing_status(
    Extension(current_user): Extension<UserAccount>,
    State(state): State<AppState>,
) -> Result<Json<RepoSigningStatusResponse>, AppError> {
    Ok(Json(
        state.service.get_repo_signing_status(&current_user).await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/signing/reconcile/progress",
    tag = "Signing",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get latest signing/unsigning reconcile progress", body = RepoSigningReconcileProgressResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn get_repo_signing_reconcile_progress(
    State(state): State<AppState>,
) -> Result<Json<RepoSigningReconcileProgressResponse>, AppError> {
    Ok(Json(
        state.service.get_repo_signing_reconcile_progress().await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/signing/config",
    tag = "Signing",
    request_body = UpdateRepoSigningConfigRequest,
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Update repository signing config", body = RepoSigningStatusResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn update_repo_signing_config(
    Extension(current_user): Extension<UserAccount>,
    State(state): State<AppState>,
    Json(request): Json<UpdateRepoSigningConfigRequest>,
) -> Result<Json<RepoSigningStatusResponse>, AppError> {
    Ok(Json(
        state
            .service
            .update_repo_signing_config(request, current_user.id)
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/signing/import",
    tag = "Signing",
    request_body = ImportRepoSigningKeyRequest,
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Import repository signing private key", body = ImportRepoSigningKeyResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn import_repo_signing_key(
    Extension(current_user): Extension<UserAccount>,
    State(state): State<AppState>,
    Json(request): Json<ImportRepoSigningKeyRequest>,
) -> Result<Json<ImportRepoSigningKeyResponse>, AppError> {
    Ok(Json(
        state
            .service
            .import_repo_signing_key(request, current_user.id)
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/signing/generate",
    tag = "Signing",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Generate daemon-managed repository private key", body = GenerateRepoSigningKeyResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn generate_repo_signing_key(
    Extension(current_user): Extension<UserAccount>,
    State(state): State<AppState>,
) -> Result<Json<GenerateRepoSigningKeyResponse>, AppError> {
    Ok(Json(
        state
            .service
            .generate_repo_signing_key(current_user.id)
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/signing/key",
    tag = "Signing",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Remove repository signing key and disable signing", body = RepoSigningStatusResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn remove_repo_signing_key(
    Extension(current_user): Extension<UserAccount>,
    State(state): State<AppState>,
) -> Result<Json<RepoSigningStatusResponse>, AppError> {
    Ok(Json(
        state
            .service
            .remove_repo_signing_key(current_user.id)
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/signing/test",
    tag = "Signing",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Run repository signing test", body = TestRepoSigningResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn test_repo_signing(
    State(state): State<AppState>,
) -> Result<Json<TestRepoSigningResponse>, AppError> {
    Ok(Json(state.service.test_repo_signing().await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/signing/export",
    tag = "Signing",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Export managed repository private signing key (bootstrap admin only)", body = ExportRepoSigningKeyResponse),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn export_repo_signing_key(
    Extension(current_user): Extension<UserAccount>,
    State(state): State<AppState>,
) -> Result<Json<ExportRepoSigningKeyResponse>, AppError> {
    Ok(Json(
        state
            .service
            .export_repo_signing_private_key(&current_user)
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/signing/export/public",
    tag = "Signing",
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Export managed repository public signing key", body = ExportRepoSigningPublicKeyResponse),
        (status = 400, body = synforge_core::api::ApiError),
        (status = 401, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn export_repo_signing_public_key(
    State(state): State<AppState>,
) -> Result<Json<ExportRepoSigningPublicKeyResponse>, AppError> {
    Ok(Json(state.service.export_repo_signing_public_key().await?))
}
