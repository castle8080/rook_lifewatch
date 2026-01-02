use thiserror::Error;

use crate::core::frame::FrameError;
use crate::core::pipeline::ProcessingError;

pub type RookLWResult<T> = Result<T, RookLWError>;

#[derive(Debug, Error)]
pub enum RookLWError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("frame error: {0}")]
    Frame(#[from] FrameError),

    #[error("processing error: {0}")]
    Processing(#[from] ProcessingError),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("camera error: {0}")]
    Camera(String),

    #[error("implementation error: {0}")]
    Implementation(String),

    #[error("other error: {0}")]
    Other(String),
}
