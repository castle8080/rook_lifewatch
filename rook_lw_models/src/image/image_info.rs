
use serde::{Deserialize, Serialize};

use super::MotionDetectionScore;
use super::Detection;

#[derive(Clone, Debug, Serialize, Deserialize)]
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
