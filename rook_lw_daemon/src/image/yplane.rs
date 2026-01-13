use std::borrow::Cow;

use super::frame::{Frame, FrameResult, FrameError};
use super::fourcc;

/// A read-only view over an 8-bit luma (Y) plane.
///
/// `stride` is in bytes. `pixel_step` is the byte increment between adjacent X
/// samples (e.g. `1` for planar Y, `2` for YUYV where Y values are at even
/// byte offsets).
pub struct YPlane<'a> {
    pub data: Cow<'a, [u8]>,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub pixel_step: usize, // 1 for planar Y, 2 for YUYV
}

impl YPlane<'_> {

    pub fn from_frame<'a>(frame: &'a dyn Frame) -> FrameResult<YPlane<'a>> {
        if frame.get_plane_count()? < 1 {
            return Err(FrameError::ProcessingError(
                "Frame has no planes".to_string(),
            ));
        }

        let pixel_format = frame.get_pixel_format()?;

        if pixel_format == fourcc::FOURCC_YUYV {
            let width = frame.get_width()?;
            let height = frame.get_height()?;
            let plane_data = frame.get_plane_data(0)?;

            return Ok(YPlane::new(
                Cow::Borrowed(plane_data),
                width,
                height,
                width * 2,
                2,
            ));
        }
        else if pixel_format == fourcc::FOURCC_MJPG {
            let decoded = image::load_from_memory_with_format(
                frame.get_plane_data(0)?,
                image::ImageFormat::Jpeg,
            )?;
            let luma = decoded.to_luma8();
            let (width, height) = luma.dimensions();

            Ok(YPlane::new(
                Cow::Owned(luma.into_raw()),
                width as usize,
                height as usize,
                width as usize,
                1,
            ))
        }
        else {
            return Err(FrameError::ProcessingError(
                format!("Unsupported pixel format: {}", pixel_format),
            ));
        }
    }
}

impl<'a> YPlane<'a> {
    pub fn new(data: Cow<'a, [u8]>, width: usize, height: usize, stride: usize, pixel_step: usize) -> Self {
        Self {
            data,
            width,
            height,
            stride,
            pixel_step,
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn stride(&self) -> usize {
        self.stride
    }

    pub fn pixel_step(&self) -> usize {
        self.pixel_step
    }

    pub fn get(&self, x: usize, y: usize) -> Option<u8> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = y
            .saturating_mul(self.stride)
            .saturating_add(x.saturating_mul(self.pixel_step));
        self.data.get(index).copied()
    }
}
