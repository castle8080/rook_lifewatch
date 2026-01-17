
use crate::events::capture_event::CaptureEvent;

use rook_lw_models::image::Detection;

#[derive(Clone, Debug)]
pub struct ImageProcessingEvent {
    pub capture_event: CaptureEvent,
    pub detections: Option<Vec<Detection>>,
}

pub type OnImageProcessingEventCallback = Box<dyn Fn(&ImageProcessingEvent) + Send + 'static>;