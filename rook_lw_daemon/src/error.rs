use thiserror::Error;

use rook_lw_image_repo::ImageRepoError;

pub type RookLWResult<T> = Result<T, RookLWError>;

#[derive(Debug, Error)]
pub enum RookLWError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("image error: {0}")]
    Image(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("camera error: {0}")]
    Camera(String),

    #[error("initialization error: {0}")]
    Initialization(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("concurrency error: {0}")]
    Concurrency(String),

    #[error("other error: {0}")]
    Other(String),
}

impl From<ImageRepoError> for RookLWError {
    fn from(err: ImageRepoError) -> Self {
        RookLWError::Database(format!("{err}"))
    }
}

impl From<image::ImageError> for RookLWError {
    fn from(err: image::ImageError) -> Self {
        RookLWError::Image(format!("{err}"))
    }
}

impl From<opencv::Error> for RookLWError {
    fn from(err: opencv::Error) -> Self {
        RookLWError::Image(format!("OpenCV error: {}", err))
    }
}

impl From<serde_json::Error> for RookLWError {
    fn from(err: serde_json::Error) -> Self {
        RookLWError::Parse(format!("JSON parse error: {}", err))
    }
}

impl From<anyhow::Error> for RookLWError {
    fn from(err: anyhow::Error) -> Self {
        RookLWError::Other(format!("Anyhow error: {}", err))
    }
}

impl<T> From<crossbeam_channel::SendError<T>> for RookLWError {
    fn from(err: crossbeam_channel::SendError<T>) -> Self {
        RookLWError::Concurrency(format!("Channel send error: {}", err))
    }
}