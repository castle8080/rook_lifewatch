use r2d2;
use chrono;
use serde_json;
use thiserror::Error;

use crate::image::frame::FrameError;

use rusqlite;

pub type RookLWResult<T> = Result<T, RookLWError>;

#[derive(Debug, Error)]
pub enum RookLWError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("frame error: {0}")]
    Frame(#[from] FrameError),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("camera error: {0}")]
    Camera(String),

    #[error("implementation error: {0}")]
    Implementation(String),

    #[error("other error: {0}")]
    Other(String),
}

impl From<rusqlite::Error> for RookLWError {
    fn from(err: rusqlite::Error) -> Self {
        RookLWError::Other(format!("SQLite error: {err}"))
    }
}

impl From<serde_json::Error> for RookLWError {
    fn from(err: serde_json::Error) -> Self {
        RookLWError::Other(format!("Serde JSON error: {err}"))
    }
}

impl From<chrono::ParseError> for RookLWError {
    fn from(err: chrono::ParseError) -> Self {
        RookLWError::Other(format!("Chrono parse error: {err}"))
    }
}

impl From<r2d2::Error> for RookLWError {
    fn from(err: r2d2::Error) -> Self {
        RookLWError::Other(format!("r2d2 pool error: {err}"))
    }
}