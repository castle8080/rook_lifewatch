use crate::RookLWResult;
use crate::image::conversions::frame_to_dynamic_image;
use crate::image::frame::{FrameSource, FrameSlot};
use crate::image::motion::YPlaneMotionDetector;
use crate::events::{CaptureEvent, ImageProcessingEvent};
use crate::prodcon::{ProducerTask, ProducerCallbacks};
use crate::tasks::image_capturer::ImageCapturer;
use crate::tasks::motion_watcher::MotionWatcher;

use crate::events::MotionDetectionEvent;

use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use std::thread::{JoinHandle, sleep, spawn};

use chrono::{DateTime, FixedOffset};
use tracing::{info, debug};

use uuid::Uuid;

pub struct ImageDiffMotionWatcher {
    frame_source: Arc<Box<dyn FrameSource + Send + Sync>>,
    motion_detect_interval: Duration,
    motion_watch_count: u32,
    motion_detector: Box<dyn YPlaneMotionDetector>,
    image_capturer: ImageCapturer,
    round_interval: Duration,
}

impl ProducerTask<ImageProcessingEvent> for ImageDiffMotionWatcher {
    fn get_producer_callbacks(&mut self) -> &mut ProducerCallbacks<ImageProcessingEvent> {
        self.image_capturer.get_producer_callbacks()
    }
}

impl ImageDiffMotionWatcher {

    pub fn new(
        frame_source: Arc<Box<dyn FrameSource + Send + Sync>>,
        motion_detect_interval: Duration,
        motion_watch_count: u32,
        motion_detector: Box<dyn YPlaneMotionDetector>,
        image_capturer: ImageCapturer,
        round_interval: Duration,
    ) -> Self {
        Self { 
            frame_source,
            motion_detect_interval,
            motion_watch_count,
            motion_detector,
            image_capturer,
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

    fn on_motion_detected(&mut self, result: MotionDetectionEvent) -> RookLWResult<()> {
        self.image_capturer.on_motion_detected(result)
    }

    fn detect_motion(&mut self) -> RookLWResult<Option<MotionDetectionEvent>> {
        // Keep a small 2-slot ring. Each slot owns its frame and caches a YPlane.
        // YU12: YPlane is a borrowed view (no copy). MJPG: YPlane owns decoded luma.
        let mut last = FrameSlot::from_frame(self.frame_source.next_frame()?)?;
        let mut last_timestamp: DateTime<FixedOffset> = chrono::Local::now().into();

        for _watch_index in 0..self.motion_watch_count {
            sleep(self.motion_detect_interval);

            let current = FrameSlot::from_frame(self.frame_source.next_frame()?)?;
            let current_timestamp: DateTime<FixedOffset> = chrono::Local::now().into();

            let timer = Instant::now();
            let motion_score = self.motion_detector.detect_motion(
                last.yplane(),
                current.yplane(),
            )?;
            let elapsed = timer.elapsed();

            if motion_score.detected {
                let event_id = Uuid::new_v4();

                info!(
                    motion_score = motion_score.score,
                    motion_detected = motion_score.detected,
                    motion_score_properties = %format!("{:?}", motion_score.properties),
                    event_id = %event_id,
                    motion_detection_time_ms = elapsed.as_millis(),
                    "Motion detected."
                );

                let mut result = MotionDetectionEvent {
                    event_id,
                    event_timestamp: current_timestamp,
                    motion_score: motion_score.clone(),
                    capture_events: Vec::new(),
                };

                let timer = Instant::now();

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

                let elapsed = timer.elapsed();
                debug!(
                    event_id = %event_id,
                    capture_count = result.capture_events.len(),
                    capture_time_ms = elapsed.as_millis(),
                    "Captured initial motion detection images"
                );

                return Ok(Some(result));
            }

            last = current;
            last_timestamp = current_timestamp;
        }

        Ok(None)
    }

}

impl MotionWatcher for ImageDiffMotionWatcher {
    fn connect(&mut self, sender: crossbeam_channel::Sender<ImageProcessingEvent>) {
        ProducerTask::connect(self, sender);
    }

    fn start(self: Box<Self>) -> JoinHandle<RookLWResult<()>> {
        ImageDiffMotionWatcher::start(*self)
    }
}
