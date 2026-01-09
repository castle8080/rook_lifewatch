use crate::core::frame::{Frame, FrameError, FrameMetadata, FrameSource, FrameResult};
use image::DynamicImage;
use std::ffi::CStr;
use std::ptr::NonNull;
use std::time::SystemTime;

mod ffi;

/// Safe RAII wrapper around `rook_lw_camera_capturer_t*`.
///
/// - Construct with `LibCameraFrameSource::new()`
/// - Automatically destroyed on `Drop`
pub struct LibCameraFrameSource {
    inner: NonNull<ffi::rook_lw_camera_capturer_t>,
}

impl LibCameraFrameSource {

    pub fn new() -> FrameResult<Self> {
        let ptr = unsafe { ffi::rook_lw_camera_capturer_create() };
        let inner = NonNull::new(ptr).ok_or_else(|| {
            FrameError::InitializationFailed(
                "rook_lw_camera_capturer_create returned null (failed to initialize)".to_string(),
            )
        })?;
        Ok(Self { inner })
    }

    pub fn camera_count(&self) -> u32 {
        unsafe {
            ffi::rook_lw_camera_capturer_get_camera_count(self.inner.as_ptr())
        }
    }

    /// Returns the camera name at `index` as an owned `String`.
    ///
    /// Returns `None` if the index is out of range or the C API returns null.
    pub fn camera_name(&self, index: u32) -> Option<String> {
        let ptr = unsafe { ffi::rook_lw_camera_capturer_get_camera_name(self.inner.as_ptr(), index) };
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
    }

    pub fn set_camera_source(&mut self, _source: &str) -> FrameResult<()> {
        let result = unsafe {
            ffi::rook_lw_camera_capturer_set_camera_source(self.inner.as_ptr(), std::ffi::CString::new(_source).unwrap().as_ptr())
        };
        if result != 0 {
            return Err(FrameError::InitializationFailed("Failed to set camera source".to_string()));
        }
        Ok(())
    }
}

impl Drop for LibCameraFrameSource {
    fn drop(&mut self) {
        unsafe {
            ffi::rook_lw_camera_capturer_destroy(self.inner.as_ptr())
        };
    }
}

impl FrameSource for LibCameraFrameSource {

    fn start(&mut self) -> FrameResult<()> {
        unsafe {
            let result = ffi::rook_lw_camera_capturer_start(self.inner.as_ptr());
            if result != 0 {
                return Err(FrameError::InitializationFailed("Failed to start camera capturer".to_string()));
            }
            Ok(())
        }
    }

    fn stop(&mut self) -> FrameResult<()> {
        unsafe {
            let result = ffi::rook_lw_camera_capturer_stop(self.inner.as_ptr());
            if result != 0 {
                return Err(FrameError::InitializationFailed("Failed to stop camera capturer".to_string()));
            }
            Ok(())
        }
    }

    fn list_sources(&mut self) -> FrameResult<Vec<String>> {
        println!("Listing libcamera sources...");
        let mut sources = Vec::new();
        for i in 0..self.camera_count() {
            if let Some(cam) = self.camera_name(i) {
                sources.push(cam);
            }
        }
        Ok(sources)
    }

    fn set_source(&mut self, source: &str) -> FrameResult<()> {
        self.set_camera_source(source)
    }

    fn next_frame(&mut self) -> FrameResult<Frame> {

        unsafe {
            let result = ffi::rook_lw_camera_capturer_acquire_frame(self.inner.as_ptr());
            if result.is_null() {
                return Err(FrameError::ProcessingError("Failed to acquire frame".to_string()));
            }
        }

        // TODO: need to change Frame and have it so a different impl of Frame can be returned
        // I want to be able to return a handle specific to libcamera.

        let img = DynamicImage::new_rgb8(1, 1);
        Ok(Frame {
            image: img,
            metadata: FrameMetadata {
                timestamp: SystemTime::now()
            },
        })
    }
}
