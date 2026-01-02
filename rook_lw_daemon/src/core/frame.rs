use image::{DynamicImage, ImageBuffer, Rgb};
use std::time::SystemTime;

pub type RgbImage = ImageBuffer<Rgb<u8>, Vec<u8>>;

#[derive(Debug, Clone)]
pub struct FrameMetadata {
    pub timestamp: SystemTime,
    pub source_id: String,
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
}

pub trait FrameSource: Send {
    fn next_frame(&mut self) -> Result<Frame, FrameError>;
}
