use crate::events::capture_event::CaptureEvent;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct StorageEvent {
    pub capture_event: CaptureEvent,
    pub image_path: PathBuf,
}
