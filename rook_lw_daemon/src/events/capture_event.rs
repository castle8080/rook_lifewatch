use uuid::Uuid;

use crate::image::motion::motion_detector::MotionDetectionScore;
use crate::image::frame::{Frame, FrameResult, FrameError};

#[derive(Clone, Debug)]
pub struct CaptureEvent {
    pub event_id: Uuid,
    pub event_timestamp: chrono::DateTime<chrono::Local>,
    pub motion_score: MotionDetectionScore,
    pub capture_index: u32,
    pub capture_timestamp: chrono::DateTime<chrono::Local>,
    pub pixel_format: u32,
    pub width: usize,
    pub height: usize,
    pub image_data: Vec<Vec<u8>>,
}

impl Frame for CaptureEvent {

    fn get_pixel_format(&self) -> FrameResult<u32> {
        Ok(self.pixel_format)
    }
    
    fn get_width(&self) -> FrameResult<usize> {
        Ok(self.width)
    }
    
    fn get_height(&self) -> FrameResult<usize> {
        Ok(self.height)
    }
    
    fn get_plane_count(&self) -> FrameResult<usize> {
        Ok(self.image_data.len())
    }
    
    fn get_plane_data(&self, idx: usize) -> FrameResult<&[u8]> {
        self.image_data
            .get(idx).map(|v| Ok(v.as_slice()))
            .unwrap_or_else(|| Err(FrameError::ProcessingError("Plane index out of bounds".to_string())))
    }
}
