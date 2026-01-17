use thiserror::Error;

use crate::image::frame::FrameError;
use rook_lw_image_repo::ImageRepoError;


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

    #[error("database error: {0}")]
    Database(String),

    #[error("other error: {0}")]
    Other(String),
}

impl From<ImageRepoError> for RookLWError {
    fn from(err: ImageRepoError) -> Self {
        RookLWError::Database(format!("{err}"))
    }
}
