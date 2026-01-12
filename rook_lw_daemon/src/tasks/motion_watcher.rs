use std::time::Duration;
use std::thread::sleep;

use crate::image::frame::{FrameSource, FrameResult};
use crate::image::frame_slot::FrameSlot;
use crate::image::yplane;
use crate::events::capture_event::CaptureEvent;
use crate::stats::rollingz::RollingZ;

use crossbeam_channel::Sender;

use tracing::{info, debug};
use tracing_subscriber::field::debug;
use uuid::Uuid;

struct MotionDetectionResult {
    pub event_id: Uuid,
    pub event_timestamp: chrono::DateTime<chrono::Local>,
    pub motion_score: f32,
    pub capture_events: Vec<CaptureEvent>,
}

pub struct MotionWatcher {
    frame_source: Box<dyn FrameSource + Send>,
    capture_event_tx: Sender<CaptureEvent>,
    motion_detect_interval: Duration,
    motion_watch_count: u32,
    motion_threshold: f32,
    capture_count: u32,
    capture_interval: Duration,
    round_interval: Duration,
    rolling_z: RollingZ,
}

impl MotionWatcher {

    pub fn new(
        frame_source: Box<dyn FrameSource + Send>,
        capture_event_tx: Sender<CaptureEvent>,
        motion_detect_interval: Duration,
        motion_watch_count: u32,
        motion_threshold: f32,
        capture_count: u32,
        capture_interval: Duration,
        round_interval: Duration,
    ) -> Self {
        Self { 
            frame_source,
            capture_event_tx,
            motion_detect_interval,
            motion_watch_count,
            motion_threshold,
            capture_count,
            capture_interval,
            round_interval,
            rolling_z: RollingZ::new(0.05), // example alpha value
        }
    }

    pub fn run(&mut self) -> FrameResult<()> {
        self.frame_source.start()?;
        loop {
            self.run_round()?;
            sleep(self.round_interval);
        }

        // self.frame_source.stop()?;
        // Ok(())
    }

    fn on_capture_event(&mut self, event: CaptureEvent) -> FrameResult<()> {
        self.capture_event_tx
            .send(event)
            .map_err(|_| crate::image::frame::FrameError::ProcessingError("capture event receiver disconnected".to_owned()))
    }

    fn run_round(&mut self) -> FrameResult<()> {
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

    fn on_motion_detected(&mut self, result: MotionDetectionResult) -> FrameResult<()> {
        let index_offset = result.capture_events.len() as u32;

        // Emit initial capture events
        for capture_event in result.capture_events {
            self.on_capture_event(capture_event)?;
        }

        for capture_index in 0..(self.capture_count-index_offset) {

            let capture_event: CaptureEvent = {
                let frame = self.frame_source.next_frame()?;
                CaptureEvent {
                    event_id: result.event_id,
                    event_timestamp: result.event_timestamp,
                    motion_score: result.motion_score,
                    capture_index: capture_index + index_offset, // offset because first images were from motion detection
                    capture_timestamp: chrono::Local::now(),
                    pixel_format: self.frame_source.get_pixel_format()?,
                    width: self.frame_source.get_width()?,
                    height: self.frame_source.get_height()?,
                    image_data: self.get_image_data(&*frame)?,
                }
            };

            self.on_capture_event(capture_event)?;

            sleep(self.capture_interval);
        }

        Ok(())
    }

    fn detect_motion(&mut self) -> FrameResult<Option<MotionDetectionResult>> {
        // Keep a small 2-slot ring. Each slot owns its frame and caches a YPlane.
        // YUYV: YPlane is a borrowed view (no copy). MJPG: YPlane owns decoded luma.
        let mut last = FrameSlot::from_frame(self.frame_source.next_frame()?)?;
        let mut last_timestamp = chrono::Local::now();

        for _watch_index in 0..self.motion_watch_count {
            sleep(self.motion_detect_interval);

            let current = FrameSlot::from_frame(self.frame_source.next_frame()?)?;
            let current_timestamp = chrono::Local::now();

            //let motion_level = yplane::get_motion_score(last.yplane(), current.yplane(), 1)?;
            
            let motion_level = yplane::get_motion_percentile(
                last.yplane(),
                current.yplane(),
                0.95,
                1,
            )?;

            let motion_level_rz = self.rolling_z.update(motion_level as f64) as f32;

            debug!(
                motion_level = motion_level,
                motion_level_rz = motion_level_rz,
                "Motion watch sample"
            );

            if motion_level_rz >= 2.0 && motion_level >= 0.02 {
                let event_id = Uuid::new_v4();

                info!(
                    motion_level = motion_level,
                    motion_level_rz = motion_level_rz,
                    event_id = %event_id,
                    "Motion detected."
                );

                let mut result = MotionDetectionResult {
                    event_id,
                    event_timestamp: current_timestamp,
                    motion_score: motion_level,
                    capture_events: Vec::new(),
                };

                // Store first image.
                result.capture_events.push(CaptureEvent {
                    event_id,
                    event_timestamp: last_timestamp,
                    motion_score: motion_level,
                    capture_index: 0,
                    capture_timestamp: last_timestamp,
                    pixel_format: self.frame_source.get_pixel_format()?,
                    width: self.frame_source.get_width()?,
                    height: self.frame_source.get_height()?,
                    image_data: self.get_image_data(&*last.frame())?,
                });

                // store second image.
                result.capture_events.push(CaptureEvent {
                    event_id,
                    event_timestamp: current_timestamp,
                    motion_score: motion_level,
                    capture_index: 1,
                    capture_timestamp: current_timestamp,
                    pixel_format: self.frame_source.get_pixel_format()?,
                    width: self.frame_source.get_width()?,
                    height: self.frame_source.get_height()?,
                    image_data: self.get_image_data(&*current.frame())?,
                });

                return Ok(Some(result));
            }

            last = current;
            last_timestamp = current_timestamp;
        }

        Ok(None)
    }

    fn get_image_data(&self, frame: &dyn crate::image::frame::Frame) -> FrameResult<Vec<Vec<u8>>> {
        let mut image_data: Vec<Vec<u8>> = Vec::new();

        for plane_index in 0..frame.get_plane_count()? {
            let plane = frame.get_plane_data(plane_index)?;
            image_data.push(plane.to_vec());
        }

        Ok(image_data)
    }

}
