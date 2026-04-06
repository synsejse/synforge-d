use axum::body::Body;
use axum::extract::{Extension, Path, State};
use axum::http::header::{self, HeaderValue};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use synforge_core::{error::SynforgeError, model::UserAccount};
use tokio_util::io::ReaderStream;

use crate::{AppError, AppState};

pub(crate) async fn repo_root() -> Result<StatusCode, AppError> {
    Err(AppError::from(anyhow::anyhow!(SynforgeError::NotFound(
        "repository root".to_string()
    ))))
}

pub(crate) async fn download_repo_file(
    user: Option<Extension<UserAccount>>,
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let resolved = state.service.resolve_repo_file_path(&path).await?;
    let metadata = tokio::fs::metadata(&resolved).await?;
    let file = tokio::fs::File::open(&resolved).await?;
    if let Some(Extension(user)) = user {
        state
            .service
            .increment_user_download_bytes(user.id, metadata.len())
            .await?;
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    Ok((headers, Body::from_stream(ReaderStream::new(file))))
}
