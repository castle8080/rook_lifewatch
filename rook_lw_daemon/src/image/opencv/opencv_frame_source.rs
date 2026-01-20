use crate::{RookLWError, RookLWResult};
use crate::image::frame::{Frame, FrameSource};
use crate::image::fourcc::FOURCC_BGR3;
use opencv::prelude::*;
use opencv::videoio::{VideoCapture, VideoCaptureTrait, VideoCaptureTraitConst, CAP_ANY};
use std::cell::RefCell;

use super::OpenCvFrame;

/// A frame source that captures images from cameras or video streams using OpenCV.
///
/// This implementation supports:
/// - Local cameras (by numeric ID, e.g., "0", "1")
/// - Network streams (RTSP, HTTP URLs)
/// - Video files (local file paths)
///
/// The source uses `RefCell` for interior mutability to allow `next_frame()` to
/// work with `&self` (as required by the `FrameSource` trait), while still being
/// able to read frames from the underlying `VideoCapture`.
pub struct OpenCvFrameSource {
    source_name: RefCell<Option<String>>,
    capture: RefCell<Option<VideoCapture>>,
    is_started: RefCell<bool>,
}

impl OpenCvFrameSource {
    /// Create a new uninitialized OpenCV frame source.
    ///
    /// Call `set_source()` to configure the capture source before use.
    pub fn new() -> RookLWResult<Self> {
        Ok(Self {
            source_name: RefCell::new(None),
            capture: RefCell::new(None),
            is_started: RefCell::new(false),
        })
    }

    /// Try to open a camera by its numeric ID.
    fn open_camera_by_id(&self, camera_id: i32) -> RookLWResult<VideoCapture> {
        let capture = VideoCapture::new(camera_id, CAP_ANY).map_err(|e| {
            RookLWError::Initialization(format!(
                "Failed to open camera {}: {}",
                camera_id, e
            ))
        })?;

        self.verify_capture_opened(&capture, &format!("Camera {} is not available", camera_id))?;
        Ok(capture)
    }

    /// Try to open a video source by URL or file path.
    fn open_video_source(&self, url: &str) -> RookLWResult<VideoCapture> {
        let capture = VideoCapture::from_file(url, CAP_ANY).map_err(|e| {
            RookLWError::Initialization(format!("Failed to open video source '{}': {}", url, e))
        })?;

        self.verify_capture_opened(&capture, &format!("Could not open video source '{}'", url))?;
        Ok(capture)
    }

    /// Verify that a VideoCapture was successfully opened.
    fn verify_capture_opened(&self, capture: &VideoCapture, error_msg: &str) -> RookLWResult<()> {
        match capture.is_opened() {
            Ok(true) => Ok(()),
            Ok(false) => Err(RookLWError::Initialization(error_msg.to_string())),
            Err(e) => Err(RookLWError::Initialization(format!(
                "Failed to check capture status: {}",
                e
            ))),
        }
    }

    /// Get the current source name, if one has been set.
    pub fn source_name(&self) -> Option<String> {
        self.source_name.borrow().clone()
    }
}

impl Default for OpenCvFrameSource {
    fn default() -> Self {
        Self::new().expect("Failed to create default OpenCvFrameSource")
    }
}

impl Drop for OpenCvFrameSource {
    fn drop(&mut self) {
        // Explicitly release the capture to avoid GStreamer warnings.
        // OpenCV's VideoCapture may use GStreamer backend which needs proper cleanup.
        if let Some(ref mut capture) = *self.capture.get_mut() {
            let _ = capture.release();
        }
    }
}

// SAFETY: `OpenCvFrameSource` is designed to be used from a single thread at a time.
// The RefCell provides interior mutability but is not thread-safe. We implement Send
// to allow the source to be transferred between threads (e.g., moved to a worker thread),
// but it must not be shared across threads concurrently.
unsafe impl Send for OpenCvFrameSource {}

impl FrameSource for OpenCvFrameSource {
    fn list_sources(&mut self) -> RookLWResult<Vec<String>> {
        // OpenCV does not provide a portable way to enumerate available cameras.
        // This would require platform-specific APIs (e.g., V4L2 on Linux, DirectShow on Windows).
        // For now, we return a stub list with common camera indices.
        //
        // TODO: Implement platform-specific camera enumeration if needed.
        Ok(vec![
            "0".to_string(),
            "1".to_string(),
        ])
    }

    fn set_source(&mut self, source: &str, required_buffer_count: u32) -> RookLWResult<()> {
        // Stop any existing capture
        if *self.is_started.get_mut() {
            self.stop()?;
        }

        // Try to parse as integer camera ID, otherwise treat as URL/file path
        let capture = if let Ok(cam_id) = source.parse::<i32>() {
            self.open_camera_by_id(cam_id)?
        } else {
            self.open_video_source(source)?
        };

        *self.source_name.get_mut() = Some(source.to_string());
        *self.capture.get_mut() = Some(capture);
        Ok(())
    }

    fn get_camera_detail(&self) -> RookLWResult<String> {
        // TODO: Implement more detailed camera information retrieval if possible.
        match self.source_name.borrow().as_ref() {
            Some(name) => Ok(format!("OpenCV Frame Source: {}", name)),
            None => Err(RookLWError::Camera("No source configured".to_string())),
        }
    }

    fn start(&mut self) -> RookLWResult<()> {
        // Verify we have a configured capture
        if self.capture.get_mut().is_none() {
            return Err(RookLWError::Initialization(
                "No source configured. Call set_source() first.".to_string(),
            ));
        }

        // OpenCV VideoCapture doesn't have an explicit start/stop mechanism.
        // Capture begins automatically when opened and frames are grabbed on demand.
        // We just track the started state for API consistency.
        *self.is_started.get_mut() = true;
        Ok(())
    }

    fn stop(&mut self) -> RookLWResult<()> {
        *self.is_started.get_mut() = false;

        // Release the capture when stopping
        if let Some(ref mut capture) = *self.capture.get_mut() {
            capture.release().map_err(|e| {
                RookLWError::Camera(format!("Failed to release capture: {}", e))
            })?;
        }
        *self.capture.get_mut() = None;

        Ok(())
    }

    fn next_frame(&self) -> RookLWResult<Box<dyn Frame + '_>> {
        // Verify capture is initialized and started
        if !*self.is_started.borrow() {
            return Err(RookLWError::Camera(
                "Frame source not started. Call start() first.".to_string(),
            ));
        }

        let mut capture_ref = self.capture.borrow_mut();
        let capture = capture_ref.as_mut().ok_or_else(|| {
            RookLWError::Camera("OpenCV capture not initialized. Call set_source() first.".to_string())
        })?;

        let mut mat = opencv::core::Mat::default();
        capture.read(&mut mat).map_err(|e| {
            RookLWError::Camera(format!("Failed to read frame: {}", e))
        })?;

        if mat.empty() {
            return Err(RookLWError::Camera("Empty frame received (end of stream or capture error)".to_string()));
        }

        let frame = OpenCvFrame::new(mat)?;
        Ok(Box::new(frame))
    }

    fn get_pixel_format(&self) -> RookLWResult<u32> {
        // OpenCV VideoCapture typically returns BGR24 format by default.
        // Individual Mat frames might be converted, but the capture delivers BGR.
        Ok(FOURCC_BGR3)
    }

    fn get_width(&self) -> RookLWResult<usize> {
        let capture_ref = self.capture.borrow();
        let capture = capture_ref.as_ref().ok_or_else(|| {
            RookLWError::Camera("OpenCV capture not initialized. Call set_source() first.".to_string())
        })?;

        let width = capture
            .get(opencv::videoio::CAP_PROP_FRAME_WIDTH)
            .map_err(|e| RookLWError::Image(format!("Failed to get width: {}", e)))?;

        if width <= 0.0 {
            return Err(RookLWError::Image(
                "Failed to get valid width from capture".to_string(),
            ));
        }
        Ok(width as usize)
    }

    fn get_height(&self) -> RookLWResult<usize> {
        let capture_ref = self.capture.borrow();
        let capture = capture_ref.as_ref().ok_or_else(|| {
            RookLWError::Camera("OpenCV capture not initialized. Call set_source() first.".to_string())
        })?;

        let height = capture
            .get(opencv::videoio::CAP_PROP_FRAME_HEIGHT)
            .map_err(|e| RookLWError::Image(format!("Failed to get height: {}", e)))?;

        if height <= 0.0 {
            return Err(RookLWError::Image(
                "Failed to get valid height from capture".to_string(),
            ));
        }
        Ok(height as usize)
    }
}
