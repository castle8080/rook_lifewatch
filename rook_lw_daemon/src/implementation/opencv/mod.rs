use crate::core::frame::{Frame, FrameError, FrameSource, FrameResult};
use opencv::prelude::*;
use opencv::videoio::{VideoCapture, VideoCaptureTrait, CAP_ANY};
use std::cell::RefCell;

pub struct OpenCvFrame {
    mat: opencv::core::Mat,
}

impl Frame for OpenCvFrame {

    fn get_plane_count(&self) -> FrameResult<usize> {
        // OpenCV Mat is typically a single plane
        Ok(1)
    }

    fn get_plane_data(&self, plane_index: usize) -> FrameResult<&[u8]> {
        if plane_index != 0 {
            return Err(FrameError::ProcessingError(
                "Invalid plane index for OpenCV frame".to_string(),
            ));
        }

        let data = self.mat.data_bytes().map_err(|e| {
            FrameError::ProcessingError(format!("Failed to get frame data: {}", e))
        })?;

        Ok(data)
    }

    fn get_pixel_format(&self) -> FrameResult<u32> {
        // OpenCV does not expose pixel format directly.
        // For simplicity, we assume BGR format (which is common in OpenCV).
        // In a real implementation, you would map OpenCV's Mat type to a pixel format.
        const PIXEL_FORMAT_BGR: u32 = 0x00000001; // Placeholder value
        Ok(PIXEL_FORMAT_BGR)
    }

    fn get_width(&self) -> FrameResult<usize> {
        Ok(self.mat.cols() as usize)
    }

    fn get_height(&self) -> FrameResult<usize> {
        Ok(self.mat.rows() as usize)
    }
}

pub struct OpencvFrameSource {
    source_name: RefCell<Option<String>>,
    capture: RefCell<Option<VideoCapture>>,
}

impl OpencvFrameSource {

    /// Try to create a new opencv frame source from the default camera (camera 0)
    pub fn new() -> FrameResult<Self> {
        Ok(Self {
            source_name: RefCell::new(None),
            capture: RefCell::new(None),
        })
    }

    /// Try to create a new opencv frame source with a specific camera ID
    pub fn set_source_id(&mut self, camera_id: i32) -> FrameResult<()> {

        let capture = VideoCapture::new(camera_id, CAP_ANY).map_err(|e| {
            FrameError::InitializationFailed(format!(
                "Failed to open camera {}: {}",
                camera_id, e
            ))
        })?;

        match capture.is_opened() {
            Ok(true) => {
                *self.source_name.get_mut() = Some(format!("opencv-camera-{}", camera_id));
                *self.capture.get_mut() = Some(capture);
                Ok(())
            }
            Ok(false) => {
                Err(FrameError::InitializationFailed(format!(
                    "Camera {} is not available",
                    camera_id
                )))
            }
            Err(e) => {
                Err(FrameError::InitializationFailed(format!(
                    "Failed to check camera {} status: {}",
                    camera_id, e
                )))
            }
        }
    }

    /// Try to create a new opencv frame source from a URL (HTTP, RTSP, etc.)
    pub fn set_source_url(&mut self, url: &str) -> FrameResult<()> {
        let capture = VideoCapture::from_file(url, CAP_ANY).map_err(|e| {
            FrameError::InitializationFailed(format!("Failed to open video URL {}: {}", url, e))
        })?;

        match capture.is_opened() {
            Ok(true) => {
                *self.source_name.get_mut() = Some(format!("opencv-url-{}", url));
                *self.capture.get_mut() = Some(capture);
                Ok(())
            }
            Ok(false) => {
                Err(FrameError::InitializationFailed(format!(
                    "Could not open url {}",
                    url
                )))
            }
            Err(e) => {
                Err(FrameError::InitializationFailed(format!(
                    "Failed to check url {} status: {}",
                    url, e
                )))
            }
        }
    }
}

impl Drop for OpencvFrameSource {
    fn drop(&mut self) {
        // Explicitly release the capture to avoid GStreamer warnings
        // OpenCV's VideoCapture uses GStreamer backend which needs proper cleanup
        match self.capture.get_mut().as_mut() {
            Some(capture) => {
                let _ = capture.release();
            }
            None => {}
        }
        *self.capture.get_mut() = None;
        *self.source_name.get_mut() = None;
    }
}

impl FrameSource for OpencvFrameSource {

    fn list_sources(&mut self) -> FrameResult<Vec<String>> {
        // OpenCV does not provide a way to list available cameras.
        // TODO: figure out some way to get these, perhaps via platform-specific APIs.
        Ok(vec!["0".to_string()])
    }

    fn set_source(&mut self, source: &str) -> FrameResult<()> {
        // Try to parse as integer camera ID
        if let Ok(cam_id) = source.parse::<i32>() {
            self.set_source_id(cam_id)
        } else {
            // Otherwise treat as URL
            self.set_source_url(source)
        }
    }

    fn start(&mut self) -> FrameResult<()> {
        // TODO
        Ok(())
    }

    fn stop(&mut self) -> FrameResult<()> {
        // TODO
        Ok(())
    }

    fn next_frame(&self) -> FrameResult<Box<dyn Frame + '_>> {
        let mut capture_ref = self.capture.borrow_mut();
        let capture = capture_ref.as_mut().ok_or_else(|| {
            FrameError::Capture("OpenCV capture not initialized. Call set_source first.".to_string())
        })?;

        let mut frame = opencv::core::Mat::default();

        capture
            .read(&mut frame)
            .map_err(|e| FrameError::Capture(format!("Failed to read frame: {}", e)))?;

        if frame.empty() {
            return Err(FrameError::Capture("Empty frame received".to_string()).into());
        }

        let opencv_frame = OpenCvFrame { mat: frame };
        Ok(Box::new(opencv_frame))
    }

    fn get_pixel_format(&self) -> FrameResult<u32> {
        // OpenCV does not expose pixel format directly.
        // For simplicity, we assume BGR format (which is common in OpenCV).
        // In a real implementation, you would map OpenCV's Mat type to a pixel format.
        const PIXEL_FORMAT_BGR: u32 = 0x00000001; // Placeholder value
        Ok(PIXEL_FORMAT_BGR)
    }
}

/*
 // code to read from data.

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
            },
        })
    }
}
    */
