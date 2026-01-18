use crate::events::capture_event::CaptureEvent;

#[derive(Clone, Debug)]
pub struct StorageEvent {
    pub capture_event: CaptureEvent,
    pub image_path: String,
}
