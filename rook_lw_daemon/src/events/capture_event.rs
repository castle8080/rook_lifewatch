use std::sync::Arc;

use image::DynamicImage;
use uuid::Uuid;

use crate::image::motion::motion_detector::MotionDetectionScore;

#[derive(Clone, Debug)]
pub struct CaptureEvent {
    pub event_id: Uuid,
    pub event_timestamp: chrono::DateTime<chrono::Local>,
    pub motion_score: MotionDetectionScore,
    pub capture_index: u32,
    pub capture_timestamp: chrono::DateTime<chrono::Local>,
    pub image: Arc<DynamicImage>,
}
