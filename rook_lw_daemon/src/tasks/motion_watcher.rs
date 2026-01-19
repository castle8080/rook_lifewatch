use crate::RookLWResult;
use crate::image::conversions::frame_to_dynamic_image;
use crate::image::frame::{FrameSource, FrameSlot};
use crate::image::motion::YPlaneMotionDetector;
use crate::events::{CaptureEvent, ImageProcessingEvent};
use crate::prodcon::{ProducerTask, ProducerCallbacks};

use rook_lw_models::image::MotionDetectionScore;

use std::sync::Arc;
use std::time::Duration;
use std::thread::{JoinHandle, sleep, spawn};

use chrono::{DateTime, FixedOffset};
use tracing::{info, debug};

use uuid::Uuid;

struct MotionDetectionResult {
    pub event_id: Uuid,
    pub event_timestamp: DateTime<FixedOffset>,
    pub motion_score: MotionDetectionScore,
    pub capture_events: Vec<CaptureEvent>,
}

pub struct MotionWatcher {
    frame_source: Box<dyn FrameSource + Send>,
    producer_callbacks: ProducerCallbacks<ImageProcessingEvent>,
    motion_detect_interval: Duration,
    motion_watch_count: u32,
    motion_detector: Box<dyn YPlaneMotionDetector>,
    capture_count: u32,
    capture_interval: Duration,
    round_interval: Duration,
}

impl ProducerTask<ImageProcessingEvent> for MotionWatcher {
    fn get_producer_callbacks(&mut self) -> &mut ProducerCallbacks<ImageProcessingEvent> {
        &mut self.producer_callbacks
    }
}

impl MotionWatcher {

    pub fn new(
        frame_source: Box<dyn FrameSource + Send>,
        motion_detect_interval: Duration,
        motion_watch_count: u32,
        motion_detector: Box<dyn YPlaneMotionDetector>,
        capture_count: u32,
        capture_interval: Duration,
        round_interval: Duration,
    ) -> Self {
        Self { 
            frame_source,
            producer_callbacks: ProducerCallbacks::new(),
            motion_detect_interval,
            motion_watch_count,
            motion_detector,
            capture_count,
            capture_interval,
            round_interval,
        }
    }

    pub fn start(mut self) -> JoinHandle<RookLWResult<()>> {
        spawn(move || {
            match self.run() {
                Ok(_) => {
                    info!("Motion watcher exiting normally");
                    Ok(())
                },
                Err(e) => {
                    info!(error = %e, "Motion watcher exiting with error");
                    Err(e)
                }
            }
        })
    }

    pub fn run(&mut self) -> RookLWResult<()> {
        info!("Starting motion watcher");
        self.frame_source.start()?;
        loop {
            self.run_round()?;
            sleep(self.round_interval);
        }

        // self.frame_source.stop()?;
        // Ok(())
    }

    fn on_image_processing_event(&mut self, event: ImageProcessingEvent) -> RookLWResult<()> {
        self.produce(event)
    }

    fn run_round(&mut self) -> RookLWResult<()> {
        match self.detect_motion()? {
            Some(motion_detection_result) => {
                self.on_motion_detected(motion_detection_result)?;
            },
            None => {
                //info!("No motion detected in this round");
            }
        }
        Ok(())
    }

    fn on_motion_detected(&mut self, result: MotionDetectionResult) -> RookLWResult<()> {
        let index_offset = result.capture_events.len() as u32;

        // Emit initial capture events
        for capture_event in result.capture_events {
            self.on_image_processing_event(ImageProcessingEvent {
                capture_event: capture_event.clone(),
                detections: None,
            })?;
        }

        for capture_index in 0..(self.capture_count-index_offset) {

            let capture_event: CaptureEvent = {
                let frame = self.frame_source.next_frame()?;
                CaptureEvent {
                    event_id: result.event_id,
                    event_timestamp: result.event_timestamp,
                    motion_score: result.motion_score.clone(),
                    capture_index: capture_index + index_offset, // offset because first images were from motion detection
                    capture_timestamp: chrono::Local::now().into(),
                    image: Arc::new(frame_to_dynamic_image(&*frame)?),
                }
            };

            self.on_image_processing_event(ImageProcessingEvent {
                capture_event: capture_event.clone(),
                detections: None,
            })?;

            sleep(self.capture_interval);
        }

        Ok(())
    }

    fn detect_motion(&mut self) -> RookLWResult<Option<MotionDetectionResult>> {
        // Keep a small 2-slot ring. Each slot owns its frame and caches a YPlane.
        // YUYV: YPlane is a borrowed view (no copy). MJPG: YPlane owns decoded luma.
        let mut last = FrameSlot::from_frame(self.frame_source.next_frame()?)?;
        let mut last_timestamp: DateTime<FixedOffset> = chrono::Local::now().into();

        for _watch_index in 0..self.motion_watch_count {
            sleep(self.motion_detect_interval);

            let current = FrameSlot::from_frame(self.frame_source.next_frame()?)?;
            let current_timestamp: DateTime<FixedOffset> = chrono::Local::now().into();

            let motion_score = self.motion_detector.detect_motion(
                last.yplane(),
                current.yplane(),
            )?;

            debug!(
                motion_level = motion_score.score,
                motion_detected = motion_score.detected,
                motion_properties = %format!("{:?}", motion_score.properties),
                "Motion watch sample"
            );

            if motion_score.detected {
                let event_id = Uuid::new_v4();

                info!(
                    motion_score = motion_score.score,
                    motion_detected = motion_score.detected,
                    motion_score_properties = %format!("{:?}", motion_score.properties),
                    event_id = %event_id,
                    "Motion detected."
                );

                let mut result = MotionDetectionResult {
                    event_id,
                    event_timestamp: current_timestamp,
                    motion_score: motion_score.clone(),
                    capture_events: Vec::new(),
                };

                // Store first image.
                result.capture_events.push(CaptureEvent {
                    event_id,
                    event_timestamp: last_timestamp,
                    motion_score: motion_score.clone(),
                    capture_index: 0,
                    capture_timestamp: last_timestamp,
                    image: Arc::new(frame_to_dynamic_image(&*last.frame())?),
                });

                // store second image.
                result.capture_events.push(CaptureEvent {
                    event_id,
                    event_timestamp: current_timestamp,
                    motion_score: motion_score.clone(),
                    capture_index: 1,
                    capture_timestamp: current_timestamp,
                    image: Arc::new(frame_to_dynamic_image(&*current.frame())?),
                });

                return Ok(Some(result));
            }

            last = current;
            last_timestamp = current_timestamp;
        }

        Ok(None)
    }

}
