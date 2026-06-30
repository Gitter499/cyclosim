use thiserror::Error;

#[derive(Debug, Error)]
pub enum RideStoreError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ride not found: {0}")]
    NotFound(String),
    #[error("invalid ride id: {0}")]
    InvalidId(String),
}

pub type Result<T> = std::result::Result<T, RideStoreError>;
