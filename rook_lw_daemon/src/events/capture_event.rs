use std::sync::Arc;

use chrono::{DateTime, FixedOffset};
use image::DynamicImage;
use uuid::Uuid;

use rook_lw_models::image::MotionDetectionScore;

#[derive(Clone, Debug)]
pub struct CaptureEvent {
    pub event_id: Uuid,
    pub event_timestamp: DateTime<FixedOffset>,
    pub motion_score: MotionDetectionScore,
    pub capture_index: u32,
    pub capture_timestamp: DateTime<FixedOffset>,
    pub image: Arc<DynamicImage>,
}
