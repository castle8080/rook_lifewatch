use std::ffi::c_void;
use std::os::raw::{c_char, c_int};

#[repr(C)]
pub struct rook_lw_camera_capturer_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct rook_lw_capture_request_t {
    _private: [u8; 0],
}

unsafe extern "C" {

    pub unsafe fn rook_lw_camera_capturer_create() -> *mut rook_lw_camera_capturer_t;

    pub unsafe fn rook_lw_camera_capturer_destroy(capturer: *mut rook_lw_camera_capturer_t);

    pub unsafe fn rook_lw_camera_capturer_get_camera_count(
        capturer: *const rook_lw_camera_capturer_t,
    ) -> u32;

    pub unsafe fn rook_lw_camera_capturer_get_camera_name(
        capturer: *const rook_lw_camera_capturer_t,
        index: u32,
    ) -> *const c_char;

    pub unsafe fn rook_lw_camera_capturer_set_camera_source(
        capturer: *mut rook_lw_camera_capturer_t,
        source: *const c_char,
    ) -> i32;

    pub unsafe fn rook_lw_camera_capturer_start(
        capturer: *mut rook_lw_camera_capturer_t,
    ) -> i32;

    pub unsafe fn rook_lw_camera_capturer_stop(
        capturer: *mut rook_lw_camera_capturer_t,
    ) -> i32;

    pub unsafe fn rook_lw_camera_capturer_acquire_frame(
        capturer: *mut rook_lw_camera_capturer_t
    ) -> *mut rook_lw_capture_request_t;

    pub unsafe fn rook_lw_capture_request_destroy(capture_request: *mut rook_lw_capture_request_t);

    pub unsafe fn rook_lw_capture_request_wait_for_completion(
        capture_request: *mut rook_lw_capture_request_t,
    ) -> i32;

    pub unsafe fn rook_lw_capture_request_get_plane_count(
        capture_request: *mut rook_lw_capture_request_t,
        out_plane_count: *mut c_int,
    ) -> i32;

    pub unsafe fn rook_lw_capture_request_get_plane_data(
        capture_request: *mut rook_lw_capture_request_t,
        plane_index: c_int,
        plane_data: *mut *mut c_void,
        plane_length: *mut usize,
    ) -> i32;
}
