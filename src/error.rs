use thiserror::Error;

#[derive(Debug, Error)]
pub enum PatchError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid diff: {0}")]
    InvalidDiff(String),

    #[error("apply failed: {0}")]
    ApplyFailed(String),

    #[error("file not found: {0}")]
    FileNotFound(String),
}
