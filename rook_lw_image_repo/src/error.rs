
use thiserror::Error;

pub type ImageRepoResult<T> = Result<T, ImageRepoError>;

#[derive(Debug, Error)]
pub enum ImageRepoError {
    #[error("I/O error: {0}")]
    Io(std::io::Error),

    #[error("database error: {0}")]
    Database(String),

    #[error("parse error: {0}")]
    Parse(String),
}

impl From<std::io::Error> for ImageRepoError {
    fn from(err: std::io::Error) -> Self {
        ImageRepoError::Io(err)
    }
}

impl From<rusqlite::Error> for ImageRepoError {
    fn from(err: rusqlite::Error) -> Self {
        ImageRepoError::Database(format!("SQLite error: {err}"))
    }
}

impl From<r2d2::Error> for ImageRepoError {
    fn from(err: r2d2::Error) -> Self {
        ImageRepoError::Database(format!("r2d2 pool error: {err}"))
    }
}

impl From<serde_json::Error> for ImageRepoError {
    fn from(err: serde_json::Error) -> Self {
        ImageRepoError::Parse(format!("Serde JSON error: {err}"))
    }
}

impl From<chrono::ParseError> for ImageRepoError {
    fn from(err: chrono::ParseError) -> Self {
        ImageRepoError::Parse(format!("Chrono parse error: {err}"))
    }
}