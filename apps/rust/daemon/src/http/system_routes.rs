use axum::extract::State;

use super::{AppError, AppState};

pub(crate) async fn healthz(State(state): State<AppState>) -> Result<&'static str, AppError> {
    state.service.health_check().await?;
    Ok("ok")
}

pub(crate) async fn readyz(State(state): State<AppState>) -> Result<&'static str, AppError> {
    state.service.health_check().await?;
    Ok("ready")
}
