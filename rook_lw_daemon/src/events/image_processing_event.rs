
use crate::events::capture_event::CaptureEvent;

use rook_lw_models::image::DetectionResult;

#[derive(Clone, Debug)]
pub struct ImageProcessingEvent {
    pub capture_event: CaptureEvent,
    pub detection_result: Option<DetectionResult>,
}

pub type OnImageProcessingEventCallback = Box<dyn Fn(&ImageProcessingEvent) + Send + 'static>;