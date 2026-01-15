use crate::events::storage_event::StorageEvent;
use crate::image::frame::FrameResult;
use crate::image::object_detection::ObjectDetector;

use crossbeam_channel::Receiver;
use image::ImageReader;
use tracing::{info, error};

pub struct ImageDetector {
    storage_event_rx: Receiver<StorageEvent>,
    //object_detector: OpenCVObjectDetector,
    object_detector: Box<dyn ObjectDetector>,
}

impl ImageDetector {
    pub fn new(storage_event_rx: Receiver<StorageEvent>, object_detector: Box<dyn ObjectDetector>) -> Self {
        Self {
            storage_event_rx,
            object_detector,
        }
    }

    pub fn run(&mut self) -> FrameResult<()> {
        while let Ok(storage_event) = self.storage_event_rx.recv() {
            if let Err(e) = self.process_storage_event(storage_event) {
                error!(error = %e, "Failed to process storage event");
            }
        }
        Ok(())
    }

    fn process_storage_event(&mut self, storage_event: StorageEvent) -> FrameResult<()> {
        let image_path = &storage_event.image_path;
        let detection_file = image_path.with_extension("detections.json");

        // Check if detection file already exists
        if detection_file.exists() {
            info!(
                image_path = %image_path.display(),
                detection_file = %detection_file.display(),
                "Detection file already exists, skipping"
            );
            return Ok(());
        }

        info!(
            event_id = %storage_event.capture_event.event_id,
            image_path = %image_path.display(),
            "Processing image for object detection"
        );

        // Load DynamicImage from jpeg file
        let img = ImageReader::open(&image_path)?
            .decode()?;
        
        info!("Image loaded, running detection");

        let detections = self.object_detector.detect(&img)?;

        info!(detection_count = detections.len(), "Detections found");

        if tracing::enabled!(tracing::Level::INFO) {
            for (i, detection) in detections.iter().enumerate() {
                info!(
                    detection_index = i,
                    class_id = detection.class_id,
                    class_name = %detection.class_name,
                    confidence = %format!("{}", detection.confidence),
                    "Detection details"
                );
            }
        }

        // Write detections to JSON file
        let json_data = serde_json::to_string_pretty(&detections)?;
        std::fs::write(&detection_file, json_data)?;

        info!(detection_file = %detection_file.display(), "Wrote detections file");

        Ok(())
    }
}
