use uuid::Uuid;
use chrono::{DateTime, FixedOffset};

use rook_lw_models::image::MotionDetectionScore;
use crate::events::capture_event::CaptureEvent;

#[derive(Clone, Debug)]
pub struct MotionDetectionEvent {
    pub event_id: Uuid,
    pub event_timestamp: DateTime<FixedOffset>,
    pub motion_score: MotionDetectionScore,
    pub capture_events: Vec<CaptureEvent>,
}
