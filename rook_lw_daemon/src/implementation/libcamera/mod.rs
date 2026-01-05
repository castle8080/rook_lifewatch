use crate::core::frame::{Frame, FrameError, FrameMetadata, FrameSource, FrameResult};
use image::DynamicImage;
use std::ffi::CStr;
use std::ptr::NonNull;
use std::time::SystemTime;

mod ffi;

/// Safe RAII wrapper around `rook_lw_camera_capturer_t*`.
///
/// - Construct with `CameraCapturer::new()`
/// - Automatically destroyed on `Drop`
pub struct CameraCapturer {
    inner: NonNull<ffi::rook_lw_camera_capturer_t>,
}

impl CameraCapturer {
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
        unsafe { ffi::rook_lw_camera_capturer_get_camera_count(self.inner.as_ptr()) }
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
}

impl Drop for CameraCapturer {
    fn drop(&mut self) {
        unsafe { ffi::rook_lw_camera_capturer_destroy(self.inner.as_ptr()) };
    }
}

pub struct LibCameraFrameSource {
    camera_id: String,
}

impl LibCameraFrameSource {
    /// Try to create a new libcamera frame source
    /// This will check if libcamera is available and accessible at runtime
    pub fn try_new(camera: Option<&str>) -> FrameResult<Self> {
        // TODO: Actual runtime check for libcamera availability
        // For now, we'll simulate a check

        let capturer = CameraCapturer::new()?;

        println!(
            "LibCamera: Found {} cameras",
            capturer.camera_count()
        );

        Ok(Self {
            camera_id: camera.unwrap_or("libcamera-0").to_string(),
        })
    }
}

impl FrameSource for LibCameraFrameSource {
    fn next_frame(&mut self) -> FrameResult<Frame> {
        let img = DynamicImage::new_rgb8(1, 1);
        Ok(Frame {
            image: img,
            metadata: FrameMetadata {
                timestamp: SystemTime::now(),
                source_id: self.camera_id.clone(),
            },
        })
    }
}
