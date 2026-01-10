use std::{fs, path};
use std::time::Duration;
use std::thread::sleep;

use crate::image::conversions::frame_to_jpeg_bytes;
use crate::image::frame::{FrameSource, FrameResult};
use crate::image::frame_slot::FrameSlot;
use crate::image::yplane;

pub struct MotionWatcher {
    frame_source: Box<dyn FrameSource>,
    motion_detect_interval: Duration,
    motion_watch_count: u32,
    motion_threshold: f32,
    capture_count: u32,
    capture_interval: Duration,
    round_interval: Duration,
}

impl MotionWatcher {

    pub fn new(
        frame_source: Box<dyn FrameSource>,
        motion_detect_interval: Duration,
        motion_watch_count: u32,
        motion_threshold: f32,
        capture_count: u32,
        capture_interval: Duration,
        round_interval: Duration,
    ) -> Self {
        Self { 
            frame_source,
            motion_detect_interval,
            motion_watch_count,
            motion_threshold,
            capture_count,
            capture_interval,
            round_interval,
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

    fn run_round(&mut self) -> FrameResult<()> {
        let motion_score = self.detect_motion()?;
        if motion_score >= self.motion_threshold {
            self.capture_images(motion_score)?;
        }
        Ok(())
    }

    fn on_captured_image(&mut self, jpeg_bytes: Vec<u8>, _motion_score: f32) -> FrameResult<()> {
        // Placeholder: In a real implementation, save to disk, send over network, etc.
        println!("Captured image of size {} bytes", jpeg_bytes.len());

        // This is temporary.
        let image_dump_dir = "var/images";

        fs::create_dir_all(image_dump_dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S%.3f");
        let image_path1 = path::Path::new(image_dump_dir).join(format!("{}.jpg", timestamp));

        fs::write(&image_path1, jpeg_bytes)?;
        Ok(())
    }

    fn capture_images(&mut self, motion_score: f32) -> FrameResult<()> {
        println!("Motion detected! motion_score: {} - Capturing {} frames...", motion_score, self.capture_count);
        for _capture_index in 0..self.capture_count {
            let jpeg_bytes = {
                let frame = self.frame_source.next_frame()?;
                frame_to_jpeg_bytes(&*frame)?
            };
            self.on_captured_image(jpeg_bytes, motion_score)?;
            sleep(self.capture_interval);
        }
        Ok(())
    }

    fn detect_motion(&mut self) -> FrameResult<f32> {
        // Keep a small 2-slot ring. Each slot owns its frame and caches a YPlane.
        // YUYV: YPlane is a borrowed view (no copy). MJPG: YPlane owns decoded luma.
        let mut last = FrameSlot::from_frame(self.frame_source.next_frame()?)?;

        for _watch_index in 0..self.motion_watch_count {
            sleep(self.motion_detect_interval);

            let current = FrameSlot::from_frame(self.frame_source.next_frame()?)?;

            let motion_level = yplane::get_motion_score(last.yplane(), current.yplane(), 1)?;
            
            if motion_level >= self.motion_threshold {
                return Ok(motion_level);
            }

            last = current;
        }

        Ok(0.0)
    }
}
