use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Clone, Debug, Error)]
pub enum AppError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("resource conflict: {0}")]
    Conflict(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
    #[error("external process error: {0}")]
    External(String),
}
