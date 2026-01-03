use crate::core::frame::{Frame, FrameError, FrameMetadata, FrameSource};
use image::{DynamicImage, ImageBuffer, Rgb};
use opencv::prelude::*;
use opencv::videoio::{VideoCapture, VideoCaptureTrait, CAP_ANY};
use std::time::SystemTime;

pub struct OpencvFrameSource {
    source_id: String,
    capture: VideoCapture,
}

impl OpencvFrameSource {
    /// Try to create a new opencv frame source from the default camera (camera 0)
    pub fn try_new(camera: Option<&str>) -> Result<Self, FrameError> {
        // Try to get from environment variable first, otherwise default to camera 0
        match camera {
            Some(cam_str) => {
                // Try to parse as integer camera ID
                if let Ok(cam_id) = cam_str.parse::<i32>() {
                    Self::try_new_with_id(cam_id)
                } else {
                    // Otherwise treat as URL
                    Self::try_new_from_url(cam_str)
                }
            }
            None => {
                Self::try_new_with_id(0)
            }
        }
    }

    /// Try to create a new opencv frame source with a specific camera ID
    pub fn try_new_with_id(camera_id: i32) -> Result<Self, FrameError> {
        let capture = VideoCapture::new(camera_id, CAP_ANY).map_err(|e| {
            FrameError::InitializationFailed(format!(
                "Failed to open camera {}: {}",
                camera_id, e
            ))
        })?;

        if !capture.is_opened().map_err(|e| {
            FrameError::InitializationFailed(format!("Failed to check camera status: {}", e))
        })? {
            return Err(FrameError::InitializationFailed(format!(
                "Camera {} is not available",
                camera_id
            )));
        }

        Ok(Self {
            source_id: format!("opencv-camera-{}", camera_id),
            capture,
        })
    }

    /// Try to create a new opencv frame source from a URL (HTTP, RTSP, etc.)
    pub fn try_new_from_url(url: &str) -> Result<Self, FrameError> {
        let capture = VideoCapture::from_file(url, CAP_ANY).map_err(|e| {
            FrameError::InitializationFailed(format!("Failed to open video URL {}: {}", url, e))
        })?;

        if !capture.is_opened().map_err(|e| {
            FrameError::InitializationFailed(format!("Failed to check video stream status: {}", e))
        })? {
            return Err(FrameError::InitializationFailed(format!(
                "Video stream {} is not available",
                url
            )));
        }

        Ok(Self {
            source_id: format!("opencv-url-{}", url),
            capture,
        })
    }
}

impl Drop for OpencvFrameSource {
    fn drop(&mut self) {
        // Explicitly release the capture to avoid GStreamer warnings
        // OpenCV's VideoCapture uses GStreamer backend which needs proper cleanup
        let _ = self.capture.release();
    }
}

impl FrameSource for OpencvFrameSource {
    fn next_frame(&mut self) -> Result<Frame, FrameError> {
        let mut frame = opencv::core::Mat::default();

        self.capture
            .read(&mut frame)
            .map_err(|e| FrameError::Capture(format!("Failed to read frame: {}", e)))?;

        if frame.empty() {
            return Err(FrameError::Capture("Empty frame received".to_string()));
        }

        // Convert BGR (OpenCV default) to RGB
        let mut rgb_frame = opencv::core::Mat::default();
        opencv::imgproc::cvt_color(
            &frame,
            &mut rgb_frame,
            opencv::imgproc::COLOR_BGR2RGB,
            0,
        )
        .map_err(|e| FrameError::Capture(format!("Failed to convert color: {}", e)))?;

        // Get frame dimensions
        let rows = rgb_frame.rows();
        let cols = rgb_frame.cols();

        if rows <= 0 || cols <= 0 {
            return Err(FrameError::Capture(format!(
                "Invalid frame dimensions: {}x{}",
                cols, rows
            )));
        }

        // Convert to image crate format
        let data = rgb_frame
            .data_bytes()
            .map_err(|e| FrameError::Capture(format!("Failed to get frame data: {}", e)))?;

        let img_buffer = ImageBuffer::<Rgb<u8>, _>::from_raw(
            cols as u32,
            rows as u32,
            data.to_vec(),
        )
        .ok_or_else(|| FrameError::Capture("Failed to create image buffer".to_string()))?;

        Ok(Frame {
            image: DynamicImage::ImageRgb8(img_buffer),
            metadata: FrameMetadata {
                timestamp: SystemTime::now(),
                source_id: self.source_id.clone(),
            },
        })
    }
}
