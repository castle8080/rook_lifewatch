use crate::RookLWResult;
use crate::events::{ImageProcessingEvent, CaptureEvent};
use crate::image::object_detection::ObjectDetector;

use crate::prodcon::{
    ProducerTask, ConsumerTask,
    ProducerCallbacks
};

use tracing::info;

pub struct ImageDetector {
    object_detector: Box<dyn ObjectDetector>,
    producer_callbacks: ProducerCallbacks<ImageProcessingEvent>,
}

impl ProducerTask<ImageProcessingEvent> for ImageDetector {
    fn get_producer_callbacks(&mut self) -> &mut ProducerCallbacks<ImageProcessingEvent> {
        &mut self.producer_callbacks
    }
}

impl ConsumerTask<ImageProcessingEvent> for ImageDetector {
    fn consume(&mut self, item: ImageProcessingEvent) -> RookLWResult<()> {
        self.process_capture_event(&item.capture_event)
    }
}

impl ImageDetector {
    pub fn new(object_detector: Box<dyn ObjectDetector>) -> Self {
        Self {
            object_detector,
            producer_callbacks: ProducerCallbacks::new(),
        }
    }

    fn process_capture_event(&mut self, capture_event: &CaptureEvent) -> RookLWResult<()> {
        info!(
            event_id = %capture_event.event_id,
            "Processing image for object detection"
        );
    
        let detections = self.object_detector.detect(&capture_event.image)?;

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

        // Only send event if there are detections
        if detections.len() > 0 {
            let image_processing_event = ImageProcessingEvent {
                detections: Some(detections),
                capture_event: capture_event.clone(),
            };
            self.produce(image_processing_event)?;
        }

        Ok(())
    }
}
