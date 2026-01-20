use std::ffi::c_void;
use std::os::raw::c_char;

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
        out_camera_count: *mut u32,
    ) -> i32;

    pub unsafe fn rook_lw_camera_capturer_get_camera_name(
        capturer: *const rook_lw_camera_capturer_t,
        index: u32,
        out_camera_name: *mut *const c_char,
    ) -> i32;

    pub unsafe fn rook_lw_camera_capturer_set_camera_source(
        capturer: *mut rook_lw_camera_capturer_t,
        source: *const c_char,
        required_buffer_count: u32,
    ) -> i32;

    pub unsafe fn rook_lw_camera_capturer_get_pixel_format(
        capturer: *const rook_lw_camera_capturer_t,
        pixel_format: *mut u32,
    ) -> i32;
    
    pub unsafe fn rook_lw_camera_capturer_get_width(
        capturer: *const rook_lw_camera_capturer_t,
        out_width: *mut u32,
    ) -> i32;

    pub unsafe fn rook_lw_camera_capturer_get_height(
        capturer: *const rook_lw_camera_capturer_t,
        out_height: *mut u32,
    ) -> i32;

    pub unsafe fn rook_lw_camera_capturer_get_stride(
        capturer: *const rook_lw_camera_capturer_t,
        out_stride: *mut u32,
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

    pub unsafe fn rook_lw_capture_request_get_status(
        capture_request: *mut rook_lw_capture_request_t,
        status: *mut i32,
    ) -> i32;

    pub unsafe fn rook_lw_capture_request_get_plane_count(
        capture_request: *mut rook_lw_capture_request_t,
        out_plane_count: *mut i32,
    ) -> i32;

    pub unsafe fn rook_lw_capture_request_get_plane_data(
        capture_request: *mut rook_lw_capture_request_t,
        plane_index: i32,
        plane_data: *mut *mut c_void,
        plane_length: *mut usize,
    ) -> i32;
}
