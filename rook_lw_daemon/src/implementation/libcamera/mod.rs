use crate::core::frame::{Frame, FrameError, FrameSource, FrameResult};
use std::ffi::CStr;
use std::os::raw::c_int;
use std::ffi::c_void;
use std::ptr::NonNull;

mod ffi;

/// Mirrors the C `CaptureRequestStatus` enum.
///
/// Keep the discriminants in sync with the C API.
#[repr(i32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CaptureRequestStatus {
    CaptureRequestInitial = 0,
    CaptureRequestPending = 1,
    CaptureRequestComplete = 2,
    CaptureRequestCancelled = 3,
}

impl CaptureRequestStatus {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::CaptureRequestInitial),
            1 => Some(Self::CaptureRequestPending),
            2 => Some(Self::CaptureRequestComplete),
            3 => Some(Self::CaptureRequestCancelled),
            _ => None,
        }
    }

    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl TryFrom<i32> for CaptureRequestStatus {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::from_i32(value).ok_or(value)
    }
}

impl TryFrom<u32> for CaptureRequestStatus {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > i32::MAX as u32 {
            return Err(value);
        }
        CaptureRequestStatus::try_from(value as i32).map_err(|_| value)
    }
}

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

    fn next_frame(&self) -> FrameResult<Box<dyn Frame + '_>> {
        unsafe {
            let result = ffi::rook_lw_camera_capturer_acquire_frame(self.inner.as_ptr());
            if result.is_null() {
                return Err(FrameError::ProcessingError("Failed to acquire frame".to_string()));
            }

            let frame = LibCameraFrame {
                inner: NonNull::new_unchecked(result),
            };

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
}

pub struct LibCameraFrame {
    inner: NonNull<ffi::rook_lw_capture_request_t>,
}

impl LibCameraFrame {

    pub fn wait_for_completion(&self) -> FrameResult<()> {
        unsafe {
            let result = ffi::rook_lw_capture_request_wait_for_completion(self.inner.as_ptr());
            if result != 0 {
                return Err(FrameError::ProcessingError(
                    "Failed to wait for capture request completion".to_string(),
                ));
            }
            Ok(())
        }
    }

    pub fn status(&self) -> FrameResult<CaptureRequestStatus> {
        unsafe {
            let mut status_code: i32 = 0;
            if ffi::rook_lw_capture_request_get_status(
                self.inner.as_ptr(),
                &mut status_code as *mut i32,
            ) != 0 {
                return Err(FrameError::ProcessingError(
                    "Failed to get capture request status".to_string(),
                ));
            }

            CaptureRequestStatus::try_from(status_code).map_err(|_| {
                FrameError::ProcessingError(format!(
                    "Unknown capture request status code: {}",
                    status_code
                ))
            })
        }
    }
}

impl Drop for LibCameraFrame {
    fn drop(&mut self) {
        unsafe {
            ffi::rook_lw_capture_request_destroy(self.inner.as_ptr())
        };
    }
}

impl Frame for LibCameraFrame {

    fn get_plane_count(&self) -> FrameResult<usize> {
        unsafe {
            let mut plane_count: c_int = 0;
            if ffi::rook_lw_capture_request_get_plane_count(
                self.inner.as_ptr(),
                &mut plane_count as *mut c_int,
            ) != 0
                || plane_count <= 0
            {
                return Err(FrameError::ProcessingError(
                    "Failed to get plane count".to_string(),
                ));
            }
            Ok(plane_count as usize)
        }
    }
    
    fn get_plane_data(&self, plane_index: usize) -> FrameResult<&[u8]> {
        unsafe {
            let mut plane_count: i32 = 0;
            if ffi::rook_lw_capture_request_get_plane_count(
                self.inner.as_ptr(),
                &mut plane_count as *mut i32,
            ) != 0
                || plane_count <= 0
            {
                return Err(FrameError::ProcessingError(
                    "Failed to get plane count".to_string(),
                ));
            }

            let mut plane_ptr: *mut c_void = std::ptr::null_mut();
            let mut plane_len: usize = 0;
            if ffi::rook_lw_capture_request_get_plane_data(
                self.inner.as_ptr(),
                plane_index as i32,
                &mut plane_ptr as *mut *mut c_void,
                &mut plane_len as *mut usize,
            ) != 0
                || plane_ptr.is_null()
            {
                return Err(FrameError::ProcessingError(
                    "Failed to get plane data".to_string(),
                ));
            }

            Ok(std::slice::from_raw_parts(plane_ptr as *const u8, plane_len))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CaptureRequestStatus;

    #[test]
    fn capture_request_status_round_trip() {
        let status = CaptureRequestStatus::CaptureRequestComplete;
        assert_eq!(CaptureRequestStatus::from_i32(status.as_i32()), Some(status));
    }

    #[test]
    fn capture_request_status_unknown_is_none() {
        assert_eq!(CaptureRequestStatus::from_i32(123), None);
    }
}
