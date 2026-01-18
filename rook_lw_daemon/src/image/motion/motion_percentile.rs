
use crate::{RookLWError, RookLWResult};
use crate::image::yplane::YPlane;

/// Compute a normalized luma difference percentile score between two Y planes.
///
/// This is a motion metric based on the distribution of sampled per-pixel luma
/// differences:
/// - `0.0` means frames are identical (for sampled pixels)
/// - `1.0` means the requested percentile of sampled pixels differs by 255
///
/// The per-pixel diffs are `d_i = |Y_1 - Y_2|` (0..=255). This function returns
/// the `percentile`-quantile of `d_i` using the nearest-rank definition:
///
/// Let $N$ be the number of sampled pixels and $p \in [0,1]$. The rank is
/// $r = \lceil pN \rceil$, clamped to `[1, N]`. The returned value is the
/// smallest `d` such that at least `r` samples are `<= d`, normalized by 255.
///
/// This is computed exactly and efficiently via a 256-bin histogram (no sorting,
/// no per-pixel storage).
///
/// `sample_step` trades accuracy for speed. For example, `sample_step = 2`
/// samples every other pixel in both X and Y.
pub fn get_motion_percentile(
    a: &YPlane<'_>,
    b: &YPlane<'_>,
    percentile: f32,
    sample_step: usize,
) -> RookLWResult<f32> {
    if a.width != b.width || a.height != b.height {
        return Err(RookLWError::Image(format!(
            "YPlane size mismatch: a={}x{}, b={}x{}",
            a.width, a.height, b.width, b.height
        )));
    }

    if percentile.is_nan() {
        return Err(RookLWError::Image(
            "percentile must not be NaN".to_string(),
        ));
    }

    if !(0.0..=1.0).contains(&percentile) {
        return Err(RookLWError::Image(
            "percentile must be in [0.0, 1.0]".to_string(),
        ));
    }

    let step = sample_step.max(1);

    let mut hist: [u64; 256] = [0; 256];
    let mut sample_count: u64 = 0;

    for y in (0..a.height).step_by(step) {
        let a_row = y
            .checked_mul(a.stride)
            .ok_or_else(|| RookLWError::Image("YPlane index overflow".to_string()))?;
        let b_row = y
            .checked_mul(b.stride)
            .ok_or_else(|| RookLWError::Image("YPlane index overflow".to_string()))?;

        for x in (0..a.width).step_by(step) {
            let a_index = a_row
                .checked_add(
                    x.checked_mul(a.pixel_step)
                        .ok_or_else(|| RookLWError::Image("YPlane index overflow".to_string()))?,
                )
                .ok_or_else(|| RookLWError::Image("YPlane index overflow".to_string()))?;
            let b_index = b_row
                .checked_add(
                    x.checked_mul(b.pixel_step)
                        .ok_or_else(|| RookLWError::Image("YPlane index overflow".to_string()))?,
                )
                .ok_or_else(|| RookLWError::Image("YPlane index overflow".to_string()))?;

            let av = *a.data.get(a_index).ok_or_else(|| {
                RookLWError::Image("YPlane access out of bounds".to_string())
            })?;
            let bv = *b.data.get(b_index).ok_or_else(|| {
                RookLWError::Image("YPlane access out of bounds".to_string())
            })?;

            let diff = av.abs_diff(bv) as usize;
            hist[diff] += 1;
            sample_count += 1;
        }
    }

    if sample_count == 0 {
        return Err(RookLWError::Image(
            "YPlane has zero samples".to_string(),
        ));
    }

    // Nearest-rank: r = ceil(p * N), clamped to [1, N]
    let mut rank = (percentile * sample_count as f32).ceil() as u64;
    rank = rank.clamp(1, sample_count);

    let mut cumulative: u64 = 0;
    for (value, count) in hist.iter().enumerate() {
        cumulative += *count;
        if cumulative >= rank {
            return Ok(value as f32 / 255.0);
        }
    }

    // Should be unreachable since hist sums to sample_count.
    Ok(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn motion_percentile_identical_is_zero() {
        let a = YPlane::new(Cow::Owned(vec![10, 20, 30, 40]), 2, 2, 2, 1);
        let b = YPlane::new(Cow::Owned(vec![10, 20, 30, 40]), 2, 2, 2, 1);
        let score = get_motion_percentile(&a, &b, 0.95, 1).unwrap();
        assert_eq!(score, 0.0);
    }

    #[test]
    fn motion_percentile_single_pixel_max_diff_is_one() {
        let a = YPlane::new(Cow::Owned(vec![0]), 1, 1, 1, 1);
        let b = YPlane::new(Cow::Owned(vec![255]), 1, 1, 1, 1);
        let score = get_motion_percentile(&a, &b, 0.5, 1).unwrap();
        assert_eq!(score, 1.0);
    }
}
