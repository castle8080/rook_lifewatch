
use crate::image::frame::{FrameError, FrameResult};
use crate::image::yplane::YPlane;

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
pub fn normalized_avg_diff(
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
