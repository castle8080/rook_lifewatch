
use crate::{RookLWResult, RookLWError};
use crate::image::frame::Frame;
use std::os::raw::c_int;
use std::ffi::c_void;
use std::ptr::NonNull;

use super::ffi;
use super::capture_request_status::CaptureRequestStatus;

pub struct LibCameraFrame {
    inner: NonNull<ffi::rook_lw_capture_request_t>,
    pixel_format: u32,
    width: u32,
    height: u32,
    stride: u32,
}

impl LibCameraFrame {

    pub fn new(ptr: *mut ffi::rook_lw_capture_request_t, pixel_format: u32, width: u32, height: u32, stride: u32) -> RookLWResult<Self> {
        let inner = NonNull::new(ptr).ok_or_else(|| {
            RookLWError::Initialization(
                "rook_lw_capture_request_create returned null (failed to initialize)".to_string(),
            )
        })?;
        Ok(Self { inner, pixel_format, width, height, stride })
    }

    pub fn wait_for_completion(&self) -> RookLWResult<()> {
        unsafe {
            let result = ffi::rook_lw_capture_request_wait_for_completion(self.inner.as_ptr());
            if result != 0 {
                return Err(RookLWError::Camera(
                    "Failed to wait for capture request completion".to_string(),
                ));
            }
            Ok(())
        }
    }

    pub fn status(&self) -> RookLWResult<CaptureRequestStatus> {
        unsafe {
            let mut status_code: i32 = 0;
            if ffi::rook_lw_capture_request_get_status(
                self.inner.as_ptr(),
                &mut status_code as *mut i32,
            ) != 0 {
                return Err(RookLWError::Camera(
                    "Failed to get capture request status".to_string(),
                ));
            }

            CaptureRequestStatus::try_from(status_code).map_err(|_| {
                RookLWError::Camera(format!(
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

    fn get_pixel_format(&self) -> RookLWResult<u32> {
        Ok(self.pixel_format)
    }

    fn get_width(&self) -> RookLWResult<usize> {
        Ok(self.width as usize)
    }

    fn get_height(&self) -> RookLWResult<usize> {
        Ok(self.height as usize)
    }

    fn get_stride(&self) -> RookLWResult<usize> {
        Ok(self.stride as usize)
    }

    fn get_plane_count(&self) -> RookLWResult<usize> {
        unsafe {
            let mut plane_count: c_int = 0;
            if ffi::rook_lw_capture_request_get_plane_count(
                self.inner.as_ptr(),
                &mut plane_count as *mut c_int,
            ) != 0
                || plane_count <= 0
            {
                return Err(RookLWError::Camera(
                    "Failed to get plane count".to_string(),
                ));
            }
            Ok(plane_count as usize)
        }
    }
    
    fn get_plane_data(&self, plane_index: usize) -> RookLWResult<&[u8]> {
        unsafe {
            let mut plane_count: i32 = 0;
            if ffi::rook_lw_capture_request_get_plane_count(
                self.inner.as_ptr(),
                &mut plane_count as *mut i32,
            ) != 0
                || plane_count <= 0
            {
                return Err(RookLWError::Camera(
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
                return Err(RookLWError::Camera(
                    "Failed to get plane data".to_string(),
                ));
            }

            Ok(std::slice::from_raw_parts(plane_ptr as *const u8, plane_len))
        }
    }
}