use crate::error::RookLWResult;

use crate::image::object_detection::Detection;
use crate::image::motion::motion_detector::MotionDetectionScore;

#[derive(Clone, Debug)]
pub struct ImageInfo {
    pub image_id: String,
    pub event_id: String,
    pub event_timestamp: chrono::DateTime<chrono::Local>,
    pub motion_score: MotionDetectionScore,
    pub capture_index: u32,
    pub capture_timestamp: chrono::DateTime<chrono::Local>,
    pub detections: Option<Vec<Detection>>,
    pub image_path: String,
}

pub trait ImageInfoRepository: Send + Sync {
    fn save_image_info(&self, info: &ImageInfo) -> RookLWResult<()>;
    fn get_image_info(&self, image_id: &str) -> RookLWResult<Option<ImageInfo>>;
}
