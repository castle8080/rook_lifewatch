use crate::{events::ImageProcessingEvent, image::conversions::dynamic_image_to_jpeg};
use crate::image::frame::FrameResult;
use crate::events::capture_event::CaptureEvent;
use crate::events::storage_event::StorageEvent;

use crossbeam_channel::{Receiver, Sender};
use tracing::error;

use std::path::PathBuf;

pub type OnImageStoredCallback = Box<dyn Fn(&StorageEvent) + Send + 'static>;

pub struct ImageStorer {
    storage_root: String,
    capture_event_rx: Receiver<ImageProcessingEvent>,
    on_image_stored: Option<OnImageStoredCallback>,
}

impl ImageStorer {
    pub fn new(storage_root: String, capture_event_rx: Receiver<ImageProcessingEvent>) -> Self {
        Self { 
            storage_root, 
            capture_event_rx,
            on_image_stored: None,
        }
    }

    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&StorageEvent) + Send + 'static,
    {
        self.on_image_stored = Some(Box::new(callback));
        self
    }

    pub fn with_sender(self, sender: Sender<StorageEvent>) -> Self {
        self.with_callback(move |storage_event| {
            if let Err(e) = sender.send(storage_event.clone()) {
                error!(error = %e, "Failed to send storage event");
            }
        })
    }

    pub fn run(&mut self) -> FrameResult<()> {
        while let Ok(image_processing_event) = self.capture_event_rx.recv() {
            self.process_capture_event(image_processing_event)?;
        }
        Ok(())
    }

    fn process_capture_event(&self, image_processing_event: ImageProcessingEvent) -> FrameResult<()> {

        let capture_event = &image_processing_event.capture_event;

        tracing::info!(
            event_id = %capture_event.event_id,
            capture_index = capture_event.capture_index,
            motion_score = %format!("{}", capture_event.motion_score),
            "Processing capture event"
        );

        let jpeg_data = dynamic_image_to_jpeg(&capture_event.image, Some(85))?;

        tracing::info!(
            event_id = %capture_event.event_id,
            capture_index = capture_event.capture_index,
            jpeg_len = jpeg_data.len(),
            "Encoded JPEG"
        );

        let image_path = self.build_image_path(&capture_event);

        if let Some(parent_dir) = image_path.parent() {
            std::fs::create_dir_all(parent_dir)?;
        }

        std::fs::write(&image_path, &jpeg_data)?;

        if let Some(detections) = &image_processing_event.detections {
            tracing::info!(
                event_id = %capture_event.event_id,
                capture_index = capture_event.capture_index,
                detection_count = detections.len(),
                image_path = %image_path.display(),
                "Stored image with detections"
            );

            let detections_file = image_path.with_extension("detections.json");
            let detections_json = serde_json::to_string_pretty(&detections)?;
            std::fs::write(&detections_file, &detections_json)?;
        }

        // Invoke the callback if one is configured
        if let Some(ref callback) = self.on_image_stored {
            let storage_event = StorageEvent {
                capture_event: capture_event.clone(),
                image_path,
            };
            callback(&storage_event);
        }

        Ok(())
    }

    fn build_image_path(&self, capture_event: &CaptureEvent) -> PathBuf {
        let date_dir = capture_event
            .capture_timestamp
            .format("%Y-%m-%d")
            .to_string();

        let timestamp = capture_event
            .capture_timestamp
            .format("%Y%m%d_%H%M%S%.3f")
            .to_string();

        // Keep a stable, fairly precise numeric format in filenames.
        let motion_score = format!("{:.9}", capture_event.motion_score.score);

        let filename = format!(
            "{timestamp}_{}_{}_{}.jpg",
            capture_event.event_id, capture_event.capture_index, motion_score
        );

        PathBuf::from(&self.storage_root).join(date_dir).join(filename)
    }
}