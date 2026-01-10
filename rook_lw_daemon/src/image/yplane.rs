use std::borrow::Cow;

use super::frame::{Frame, FrameResult, FrameError};
use super::fourcc;

/// A read-only view over an 8-bit luma (Y) plane.
///
/// `stride` is in bytes. `pixel_step` is the byte increment between adjacent X
/// samples (e.g. `1` for planar Y, `2` for YUYV where Y values are at even
/// byte offsets).
pub struct YPlane<'a> {
    data: Cow<'a, [u8]>,
    width: usize,
    height: usize,
    stride: usize,
    pixel_step: usize, // 1 for planar Y, 2 for YUYV
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

/// Compute a normalized luma difference score between two Y planes.
///
/// This is intended as a simple motion metric:
/// - `0.0` means frames are identical (for sampled pixels)
/// - `1.0` means every sampled pixel differs by 255
///
/// The score is the mean absolute difference (MAD) of sampled luma pixels:
/// $$\mathrm{score} = \frac{1}{N}\sum |Y_1 - Y_2|\,/\,255$$
///
/// `sample_step` trades accuracy for speed. For example, `sample_step = 2`
/// samples every other pixel in both X and Y.
pub fn get_motion_score(
    a: &YPlane<'_>,
    b: &YPlane<'_>,
    sample_step: usize,
) -> FrameResult<f32> {
    if a.width != b.width || a.height != b.height {
        return Err(FrameError::ProcessingError(format!(
            "YPlane size mismatch: a={}x{}, b={}x{}",
            a.width, a.height, b.width, b.height
        )));
    }

    let step = sample_step.max(1);

    let mut sum_abs_diff: u64 = 0;
    let mut sample_count: u64 = 0;

    for y in (0..a.height).step_by(step) {
        let a_row = y
            .checked_mul(a.stride)
            .ok_or_else(|| FrameError::ProcessingError("YPlane index overflow".to_string()))?;
        let b_row = y
            .checked_mul(b.stride)
            .ok_or_else(|| FrameError::ProcessingError("YPlane index overflow".to_string()))?;

        for x in (0..a.width).step_by(step) {
            let a_index = a_row
                .checked_add(
                    x.checked_mul(a.pixel_step)
                        .ok_or_else(|| FrameError::ProcessingError("YPlane index overflow".to_string()))?,
                )
                .ok_or_else(|| FrameError::ProcessingError("YPlane index overflow".to_string()))?;
            let b_index = b_row
                .checked_add(
                    x.checked_mul(b.pixel_step)
                        .ok_or_else(|| FrameError::ProcessingError("YPlane index overflow".to_string()))?,
                )
                .ok_or_else(|| FrameError::ProcessingError("YPlane index overflow".to_string()))?;

            let av = *a.data.get(a_index).ok_or_else(|| {
                FrameError::ProcessingError("YPlane access out of bounds".to_string())
            })?;
            let bv = *b.data.get(b_index).ok_or_else(|| {
                FrameError::ProcessingError("YPlane access out of bounds".to_string())
            })?;

            sum_abs_diff += av.abs_diff(bv) as u64;
            sample_count += 1;
        }
    }

    if sample_count == 0 {
        return Err(FrameError::ProcessingError(
            "YPlane has zero samples".to_string(),
        ));
    }

    Ok(sum_abs_diff as f32 / (sample_count as f32 * 255.0))
}

