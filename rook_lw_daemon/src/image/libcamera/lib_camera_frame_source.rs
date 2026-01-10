use std::ffi::CStr;
use std::ptr::NonNull;

use super::super::frame::{Frame, FrameError, FrameSource, FrameResult};
use super::ffi;
use super::lib_camera_frame::LibCameraFrame;
use super::capture_request_status::CaptureRequestStatus;

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

    pub fn camera_count(&self) -> FrameResult<u32> {
        unsafe {
            let mut count: u32 = 0;
            if ffi::rook_lw_camera_capturer_get_camera_count(self.inner.as_ptr(), &mut count as *mut u32) != 0 {
                return Err(FrameError::ProcessingError("Failed to get camera count".to_string()));
            }
            Ok(count)
        }
    }

    /// Returns the camera name at `index` as an owned `String`.
    ///
    /// Returns `None` if the index is out of range or the C API returns null.
    pub fn camera_name(&self, index: u32) -> FrameResult<String> {
        let ptr = unsafe {
            let mut out_camera_name: *const std::os::raw::c_char = std::ptr::null();
            if ffi::rook_lw_camera_capturer_get_camera_name(self.inner.as_ptr(), index, &mut out_camera_name as *mut *const std::os::raw::c_char) != 0 {
                return Err(FrameError::ProcessingError("Failed to get camera name".to_string()));
            }
            out_camera_name
        };
        if ptr.is_null() {
            return Err(FrameError::ProcessingError("Camera name pointer is null".to_string()));
        }
        Ok(unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
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

    pub fn get_width(&self) -> FrameResult<u32> {
        unsafe {
            let mut width: u32 = 0;
            let result = ffi::rook_lw_camera_capturer_get_width(
                self.inner.as_ptr(),
                &mut width as *mut u32,
            );
            if result != 0 {
                return Err(FrameError::ProcessingError("Failed to get width".to_string()));
            }
            Ok(width)
        }
    }

    pub fn get_height(&self) -> FrameResult<u32> {
        unsafe {
            let mut height: u32 = 0;
            let result = ffi::rook_lw_camera_capturer_get_height(
                self.inner.as_ptr(),
                &mut height as *mut u32,
            );
            if result != 0 {
                return Err(FrameError::ProcessingError("Failed to get height".to_string()));
            }
            Ok(height)
        }
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
        let mut sources = Vec::new();
        for i in 0..self.camera_count()? {
            let cam = self.camera_name(i)?;
            sources.push(cam);
        }
        Ok(sources)
    }

    fn set_source(&mut self, source: &str) -> FrameResult<()> {
        self.set_camera_source(source)
    }

    fn next_frame(&self) -> FrameResult<Box<dyn Frame + '_>> {
        unsafe {
            let result = ffi::rook_lw_camera_capturer_acquire_frame(self.inner.as_ptr());
            if result.is_null() {
                return Err(FrameError::ProcessingError("Failed to acquire frame".to_string()));
            }

            let frame = LibCameraFrame::new(
                result,
                self.get_pixel_format()?,
                self.get_width()?,
                self.get_height()?,
            )?;

            frame.wait_for_completion()?;

            let status = frame.status()?;
            if status != CaptureRequestStatus::CaptureRequestComplete {
                return Err(FrameError::ProcessingError(format!(
                    "Capture request did not complete successfully: {:?}", status
                )));
            } 

            Ok(Box::new(frame))
        }
    }

    fn get_pixel_format(&self) -> FrameResult<u32> {
        unsafe {
            let mut pixel_format: u32 = 0;
            let result = ffi::rook_lw_camera_capturer_get_pixel_format(
                self.inner.as_ptr(),
                &mut pixel_format as *mut u32,
            );
            if result != 0 {
                return Err(FrameError::ProcessingError("Failed to get pixel format".to_string()));
            }
            Ok(pixel_format)
        }
    }

    fn get_width(&self) -> FrameResult<usize> {
        Ok(self.get_width()? as usize)
    }

    fn get_height(&self) -> FrameResult<usize> {
        Ok(self.get_height()? as usize)
    }
}
