use crate::events::{ImageProcessingEvent, CaptureEvent, OnImageProcessingEventCallback};
use crate::image::frame::FrameResult;
use crate::image::object_detection::ObjectDetector;

use crossbeam_channel::{Receiver, Sender};
use tracing::{info, error};

pub struct ImageDetector {
    image_processing_event_rx: Receiver<ImageProcessingEvent>,
    object_detector: Box<dyn ObjectDetector>,
    on_image_processing_event: Option<OnImageProcessingEventCallback>,
}

impl ImageDetector {
    pub fn new(image_processing_event_rx: Receiver<ImageProcessingEvent>, object_detector: Box<dyn ObjectDetector>) -> Self {
        Self {
            image_processing_event_rx,
            object_detector,
            on_image_processing_event: None,
        }
    }

    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&ImageProcessingEvent) + Send + 'static,
    {
        self.on_image_processing_event = Some(Box::new(callback));
        self
    }

    pub fn with_sender(self, sender: Sender<ImageProcessingEvent>) -> Self {
        self.with_callback(move |image_processing_event| {
            if let Err(e) = sender.send(image_processing_event.clone()) {
                error!(error = %e, "Failed to send image processing event");
            }
        })
    }

    pub fn run(&mut self) -> FrameResult<()> {
        while let Ok(image_processing_event) = self.image_processing_event_rx.recv() {
            if let Err(e) = self.process_capture_event(&image_processing_event.capture_event) {
                error!(error = %e, "Failed to process capture event");
            }
        }
        Ok(())
    }

    fn process_capture_event(&mut self, capture_event: &CaptureEvent) -> FrameResult<()> {
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
            if let Some(callback) = &self.on_image_processing_event {
                let image_processing_event = ImageProcessingEvent {
                    detections: Some(detections),
                    capture_event: capture_event.clone(),
                };
                callback(&image_processing_event);
            }
        }

        Ok(())
    }
}
