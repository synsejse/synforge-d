use thiserror::Error;

#[derive(Debug, Error)]
pub enum SynforgeError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("spec error: {0}")]
    Spec(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("invalid request: {0}")]
    BadRequest(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error("daemon setup is not complete")]
    SetupIncomplete,
}
