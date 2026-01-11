use crate::image::conversions::capture_event_to_jpeg;
use crate::image::frame::FrameResult;
use crate::events::capture_event::CaptureEvent;

use crossbeam_channel::Receiver;

use std::path::PathBuf;

pub struct ImageStorer {
    storage_root: String,
    capture_event_rx: Receiver<CaptureEvent>,
}

impl ImageStorer {
    pub fn new(storage_root: String, capture_event_rx: Receiver<CaptureEvent>) -> Self {
        Self { storage_root, capture_event_rx }
    }

    pub fn run(&mut self) -> FrameResult<()> {
        while let Ok(capture_event) = self.capture_event_rx.recv() {
            self.process_capture_event(capture_event)?;
        }
        Ok(())
    }

    fn process_capture_event(&self, capture_event: CaptureEvent) -> FrameResult<()> {

        tracing::info!(
            event_id = %capture_event.event_id,
            capture_index = capture_event.capture_index,
            motion_score = capture_event.motion_score,
            width = capture_event.width,
            height = capture_event.height,
            "Processing capture event"
        );

        let jpeg_data = capture_event_to_jpeg(&capture_event)?;

        let image_path = self.build_image_path(&capture_event);

        if let Some(parent_dir) = image_path.parent() {
            std::fs::create_dir_all(parent_dir)?;
        }

        std::fs::write(&image_path, jpeg_data.as_ref())?;

        Ok(())
    }

    fn build_image_path(&self, capture_event: &CaptureEvent) -> PathBuf {
        // <storage_root>/YYYY-MM-DD/YYYYMMDD_HHMMSS.sss_<event_id>_<capture_index>_<motion_score>.jpg

        let date_dir = capture_event
            .capture_timestamp
            .format("%Y-%m-%d")
            .to_string();

        let timestamp = capture_event
            .capture_timestamp
            .format("%Y%m%d_%H%M%S%.3f")
            .to_string();

        // Keep a stable, fairly precise numeric format in filenames.
        let motion_score = format!("{:.9}", capture_event.motion_score);

        let filename = format!(
            "{timestamp}_{}_{}_{}.jpg",
            capture_event.event_id, capture_event.capture_index, motion_score
        );

        PathBuf::from(&self.storage_root).join(date_dir).join(filename)
    }
}