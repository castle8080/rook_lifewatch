use crate::core::frame::{Frame, FrameError, FrameMetadata, FrameSource};
use image::DynamicImage;
use std::time::SystemTime;

pub struct StubPiFrameSource;
impl FrameSource for StubPiFrameSource {
    fn next_frame(&mut self) -> Result<Frame, FrameError> {
        let img = DynamicImage::new_rgb8(1, 1);
        Ok(Frame {
            image: img,
            metadata: FrameMetadata {
                timestamp: SystemTime::now(),
                source_id: "pi-camera-0".to_string(),
            },
        })
    }
}
