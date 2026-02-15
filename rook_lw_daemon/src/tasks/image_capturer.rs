use std::sync::Arc;
use std::thread::sleep;

use crate::RookLWResult;
use crate::events::{CaptureEvent, ImageProcessingEvent, MotionDetectionEvent};
use crate::image::conversions::frame_to_dynamic_image;
use crate::image::frame::FrameSource;
use crate::prodcon::ProducerCallbacks;

pub struct ImageCapturer {
    frame_source: Arc<Box<dyn FrameSource + Send + Sync>>,
    producer_callbacks: ProducerCallbacks<ImageProcessingEvent>,
    capture_count: u32,
    capture_interval: std::time::Duration,
}

impl ImageCapturer {
    pub fn new(frame_source: Arc<Box<dyn FrameSource + Send + Sync>>, capture_count: u32, capture_interval: std::time::Duration) -> Self {
        Self {
            frame_source,
            producer_callbacks: ProducerCallbacks::new(),
            capture_count,
            capture_interval,
        }
    }

    pub fn get_producer_callbacks(&mut self) -> &mut ProducerCallbacks<ImageProcessingEvent> {
        &mut self.producer_callbacks
    }

    pub fn on_image_processing_event(&mut self, event: ImageProcessingEvent) -> RookLWResult<()> {
        self.producer_callbacks.produce(&event)
    }

    pub fn on_motion_detected(&mut self, result: MotionDetectionEvent) -> RookLWResult<()> {
        let index_offset = result.capture_events.len() as u32;

        // Emit initial capture events
        for capture_event in result.capture_events {
            self.on_image_processing_event(ImageProcessingEvent {
                capture_event: capture_event.clone(),
                detection_result: None,
            })?;
        }

        for capture_index in 0..(self.capture_count-index_offset) {

            let image = {
                let frame = self.frame_source.next_frame()?;
                Arc::new(frame_to_dynamic_image(&*frame)?)
            };
            
            let capture_event = CaptureEvent {
                event_id: result.event_id,
                event_timestamp: result.event_timestamp,
                motion_score: result.motion_score.clone(),
                capture_index: capture_index + index_offset, // offset because first images were from motion detection
                capture_timestamp: chrono::Local::now().into(),
                image,
            };

            self.on_image_processing_event(ImageProcessingEvent {
                capture_event: capture_event.clone(),
                detection_result: None,
            })?;

            sleep(self.capture_interval);
        }

        Ok(())
    }
}