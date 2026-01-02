use crate::core::frame::{Frame, FrameError, FrameMetadata, FrameSource};
use image::{DynamicImage, ImageBuffer, Rgb};
use opencv::prelude::*;
use opencv::videoio::{VideoCapture, VideoCaptureTrait, CAP_ANY};
use std::time::SystemTime;

pub struct OpencvFrameSource {
    camera_id: i32,
    capture: VideoCapture,
}

impl OpencvFrameSource {
    /// Try to create a new opencv frame source
    /// This will check if opencv is available and a camera is accessible at runtime
    pub fn try_new() -> Result<Self, FrameError> {
        Self::try_new_with_id(0)
    }

    /// Try to create a new opencv frame source with a specific camera ID
    pub fn try_new_with_id(camera_id: i32) -> Result<Self, FrameError> {
        let capture = VideoCapture::new(camera_id, CAP_ANY)
            .map_err(|e| FrameError::InitializationFailed(format!("Failed to open camera {}: {}", camera_id, e)))?;

        if !capture.is_opened()
            .map_err(|e| FrameError::InitializationFailed(format!("Failed to check camera status: {}", e)))? {
            return Err(FrameError::InitializationFailed(format!("Camera {} is not available", camera_id)));
        }

        Ok(Self {
            camera_id,
            capture,
        })
    }
}

impl FrameSource for OpencvFrameSource {
    fn next_frame(&mut self) -> Result<Frame, FrameError> {
        let mut frame = opencv::core::Mat::default();
        
        self.capture.read(&mut frame)
            .map_err(|e| FrameError::Capture(format!("Failed to read frame: {}", e)))?;

        if frame.empty() {
            return Err(FrameError::Capture("Empty frame received".to_string()));
        }

        // Convert BGR (OpenCV default) to RGB
        let mut rgb_frame = opencv::core::Mat::default();
        opencv::imgproc::cvt_color(&frame, &mut rgb_frame, opencv::imgproc::COLOR_BGR2RGB, 0)
            .map_err(|e| FrameError::Capture(format!("Failed to convert color: {}", e)))?;

        // Get frame dimensions
        let rows = rgb_frame.rows();
        let cols = rgb_frame.cols();
        
        if rows <= 0 || cols <= 0 {
            return Err(FrameError::Capture(format!("Invalid frame dimensions: {}x{}", cols, rows)));
        }

        // Convert to image crate format
        let data = rgb_frame.data_bytes()
            .map_err(|e| FrameError::Capture(format!("Failed to get frame data: {}", e)))?;

        let img_buffer = ImageBuffer::<Rgb<u8>, _>::from_raw(
            cols as u32,
            rows as u32,
            data.to_vec(),
        ).ok_or_else(|| FrameError::Capture("Failed to create image buffer".to_string()))?;

        Ok(Frame {
            image: DynamicImage::ImageRgb8(img_buffer),
            metadata: FrameMetadata {
                timestamp: SystemTime::now(),
                source_id: format!("opencv-camera-{}", self.camera_id),
            },
        })
    }
}
