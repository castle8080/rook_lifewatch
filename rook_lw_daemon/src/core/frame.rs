use image::{DynamicImage, ImageBuffer, Rgb};
use std::time::SystemTime;

pub type RgbImage = ImageBuffer<Rgb<u8>, Vec<u8>>;

#[derive(Debug, Clone)]
pub struct FrameMetadata {
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub image: DynamicImage,
    pub metadata: FrameMetadata,
}

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("capture error: {0}")]
    Capture(String),
    #[error("no frame source implementation available")]
    NoImplementationAvailable,
    #[error("failed to initialize frame source: {0}")]
    InitializationFailed(String),
    #[error("processing error: {0}")]
    ProcessingError(String),
}

pub type FrameResult<T> = Result<T, FrameError>;

pub trait FrameSource {
    fn start(&mut self) -> FrameResult<()>;

    fn stop(&mut self) -> FrameResult<()>;

    fn next_frame(&mut self) -> FrameResult<Frame>;

    fn list_sources(&mut self) -> FrameResult<Vec<String>>;

    fn set_source(&mut self, source: &str) -> FrameResult<()>;
}
