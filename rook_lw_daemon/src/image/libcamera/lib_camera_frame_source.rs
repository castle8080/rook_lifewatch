use std::ffi::CStr;
use std::ptr::NonNull;

use crate::{RookLWResult, RookLWError};
use crate::image::frame::{Frame, FrameSource};
use super::ffi;
use super::LibCameraFrame;
use super::CaptureRequestStatus;

/// Safe RAII wrapper around `rook_lw_camera_capturer_t*`.
///
/// - Construct with `LibCameraFrameSource::new()`
/// - Automatically destroyed on `Drop`
pub struct LibCameraFrameSource {
    inner: NonNull<ffi::rook_lw_camera_capturer_t>,
}

// SAFETY: `LibCameraFrameSource` provides exclusive ownership of the underlying
// `rook_lw_camera_capturer_t*` and only exposes it through `&self`/`&mut self`
// methods. It is intended to be used from a single thread at a time, but it is
// safe to *transfer ownership* to another thread.
unsafe impl Send for LibCameraFrameSource {}

impl LibCameraFrameSource {

    pub fn new() -> RookLWResult<Self> {
        let ptr = unsafe { ffi::rook_lw_camera_capturer_create() };
        let inner = NonNull::new(ptr).ok_or_else(|| {
            RookLWError::Initialization(
                "rook_lw_camera_capturer_create returned null (failed to initialize)".to_string(),
            )
        })?;
        Ok(Self { inner })
    }

    pub fn camera_count(&self) -> RookLWResult<u32> {
        unsafe {
            let mut count: u32 = 0;
            if ffi::rook_lw_camera_capturer_get_camera_count(self.inner.as_ptr(), &mut count as *mut u32) != 0 {
                return Err(RookLWError::Camera("Failed to get camera count".to_string()));
            }
            Ok(count)
        }
    }

    /// Returns the camera name at `index` as an owned `String`.
    ///
    /// Returns `None` if the index is out of range or the C API returns null.
    pub fn camera_name(&self, index: u32) -> RookLWResult<String> {
        let ptr = unsafe {
            let mut out_camera_name: *const std::os::raw::c_char = std::ptr::null();
            if ffi::rook_lw_camera_capturer_get_camera_name(self.inner.as_ptr(), index, &mut out_camera_name as *mut *const std::os::raw::c_char) != 0 {
                return Err(RookLWError::Camera("Failed to get camera name".to_string()));
            }
            out_camera_name
        };
        if ptr.is_null() {
            return Err(RookLWError::Camera("Camera name pointer is null".to_string()));
        }
        Ok(unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
    }

    pub fn set_camera_source(&mut self, _source: &str, required_buffer_count: u32) -> RookLWResult<()> {
        let result = unsafe {
            ffi::rook_lw_camera_capturer_set_camera_source(
                self.inner.as_ptr(), 
                std::ffi::CString::new(_source).unwrap().as_ptr(),
                required_buffer_count)
        };
        if result != 0 {
            return Err(RookLWError::Camera("Failed to set camera source".to_string()));
        }
        Ok(())
    }

    pub fn get_width(&self) -> RookLWResult<u32> {
        unsafe {
            let mut width: u32 = 0;
            let result = ffi::rook_lw_camera_capturer_get_width(
                self.inner.as_ptr(),
                &mut width as *mut u32,
            );
            if result != 0 {
                return Err(RookLWError::Camera("Failed to get width".to_string()));
            }
            Ok(width)
        }
    }

    pub fn get_height(&self) -> RookLWResult<u32> {
        unsafe {
            let mut height: u32 = 0;
            let result = ffi::rook_lw_camera_capturer_get_height(
                self.inner.as_ptr(),
                &mut height as *mut u32,
            );
            if result != 0 {
                return Err(RookLWError::Image("Failed to get height".to_string()));
            }
            Ok(height)
        }
    }

    pub fn get_stride(&self) -> RookLWResult<u32> {
        unsafe {
            let mut stride: u32 = 0;
            let result = ffi::rook_lw_camera_capturer_get_stride(
                self.inner.as_ptr(),
                &mut stride as *mut u32,
            );
            if result != 0 {
                return Err(RookLWError::Camera("Failed to get stride".to_string()));
            }
            Ok(stride)
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

    fn start(&mut self) -> RookLWResult<()> {
        unsafe {
            let result = ffi::rook_lw_camera_capturer_start(self.inner.as_ptr());
            if result != 0 {
                return Err(RookLWError::Camera("Failed to start camera capturer".to_string()));
            }
            Ok(())
        }
    }

    fn stop(&mut self) -> RookLWResult<()> {
        unsafe {
            let result = ffi::rook_lw_camera_capturer_stop(self.inner.as_ptr());
            if result != 0 {
                return Err(RookLWError::Camera("Failed to stop camera capturer".to_string()));
            }
            Ok(())
        }
    }

    fn list_sources(&mut self) -> RookLWResult<Vec<String>> {
        let mut sources = Vec::new();
        for i in 0..self.camera_count()? {
            let cam = self.camera_name(i)?;
            sources.push(cam);
        }
        Ok(sources)
    }

    fn set_source(&mut self, source: &str, required_buffer_count: u32) -> RookLWResult<()> {
        self.set_camera_source(source, required_buffer_count)
    }

    fn get_camera_detail(&self) -> RookLWResult<String> {
        unsafe {
            let mut out_camera_detail: *const std::os::raw::c_char = std::ptr::null();
            let result = ffi::rook_lw_camera_capturer_get_camera_detail(
                self.inner.as_ptr(),
                &mut out_camera_detail as *mut *const std::os::raw::c_char,
            );
            if result != 0 {
                return Err(RookLWError::Camera("Failed to get camera detail".to_string()));
            }
            if out_camera_detail.is_null() {
                return Err(RookLWError::Camera("Camera detail pointer is null".to_string()));
            }
            let detail = CStr::from_ptr(out_camera_detail).to_string_lossy().into_owned();
            
            // Free allocated string from C API.
            libc::free(out_camera_detail as *mut libc::c_void);

            Ok(detail)
        }
    }

    fn next_frame(&self) -> RookLWResult<Box<dyn Frame + '_>> {
        unsafe {
            let result = ffi::rook_lw_camera_capturer_acquire_frame(self.inner.as_ptr());
            if result.is_null() {
                return Err(RookLWError::Camera("Failed to acquire frame".to_string()));
            }

            let frame = LibCameraFrame::new(
                result,
                self.get_pixel_format()?,
                self.get_width()?,
                self.get_height()?,
                self.get_stride()?,
            )?;

            frame.wait_for_completion()?;

            let status = frame.status()?;
            if status != CaptureRequestStatus::CaptureRequestComplete {
                return Err(RookLWError::Camera(format!(
                    "Capture request did not complete successfully: {:?}", status
                )));
            } 

            Ok(Box::new(frame))
        }
    }

    fn get_pixel_format(&self) -> RookLWResult<u32> {
        unsafe {
            let mut pixel_format: u32 = 0;
            let result = ffi::rook_lw_camera_capturer_get_pixel_format(
                self.inner.as_ptr(),
                &mut pixel_format as *mut u32,
            );
            if result != 0 {
                return Err(RookLWError::Camera("Failed to get pixel format".to_string()));
            }
            Ok(pixel_format)
        }
    }

    fn get_width(&self) -> RookLWResult<usize> {
        Ok(self.get_width()? as usize)
    }

    fn get_height(&self) -> RookLWResult<usize> {
        Ok(self.get_height()? as usize)
    }
}
