
use crate::events::capture_event::CaptureEvent;
use crate::image::object_detection::Detection;

#[derive(Clone, Debug)]
pub struct ImageProcessingEvent {
    pub capture_event: CaptureEvent,
    pub detections: Option<Vec<Detection>>,
}

pub type OnImageProcessingEventCallback = Box<dyn Fn(&ImageProcessingEvent) + Send + 'static>;