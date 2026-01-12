use uuid::Uuid;

use crate::image::motion::motion_detector::MotionDetectionScore;

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
