use std::os::raw::{c_char, c_int};

#[repr(C)]
pub struct rook_lw_camera_capturer_t {
    _private: [u8; 0],
}

unsafe extern "C" {
    #[allow(dead_code)]
    pub fn rook_lw_list_cameras(out_ids: *mut *mut *mut c_char, out_count: *mut u32) -> c_int;

    #[allow(dead_code)]
    pub fn rook_lw_free_camera_id_list(ids: *mut *mut c_char, count: u32);

    pub fn rook_lw_camera_capturer_create() -> *mut rook_lw_camera_capturer_t;

    pub fn rook_lw_camera_capturer_destroy(capturer: *mut rook_lw_camera_capturer_t);

    pub fn rook_lw_camera_capturer_get_camera_count(
        capturer: *const rook_lw_camera_capturer_t,
    ) -> u32;

    pub fn rook_lw_camera_capturer_get_camera_name(
        capturer: *const rook_lw_camera_capturer_t,
        index: u32,
    ) -> *const c_char;
}
