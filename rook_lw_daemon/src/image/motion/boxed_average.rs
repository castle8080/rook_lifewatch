
use crate::{RookLWError, RookLWResult};
use crate::image::yplane::YPlane;

/// Compute per-box average luma differences between two Y planes.
///
/// Divides each image into a grid of `divisions x divisions` boxes and computes
/// the average luma value in each box. Returns a vector of normalized absolute
/// differences between corresponding box averages.
///
/// # Arguments
/// * `a` - First Y plane
/// * `b` - Second Y plane
/// * `divisions` - Number of divisions along each axis (e.g., 10 creates a 10x10 grid)
///
/// # Returns
/// A vector of length `divisions * divisions`, where each element is the normalized
/// absolute difference between box averages in the range [0.0, 1.0]:
/// - `0.0` means the box averages are identical
/// - `1.0` means the box averages differ by 255
///
/// Boxes are ordered row-major: box 0 is top-left, box `divisions-1` is top-right,
/// box `divisions*divisions-1` is bottom-right.
pub fn compute_boxed_averages(
    a: &YPlane<'_>,
    b: &YPlane<'_>,
    divisions: usize,
) -> RookLWResult<Vec<f32>> {
    if a.width != b.width || a.height != b.height {
        return Err(RookLWError::Image(format!(
            "YPlane size mismatch: a={}x{}, b={}x{}",
            a.width, a.height, b.width, b.height
        )));
    }

    if divisions == 0 {
        return Err(RookLWError::Image(
            "divisions must be greater than 0".to_string(),
        ));
    }

    let box_width = a.width / divisions;
    let box_height = a.height / divisions;

    if box_width == 0 || box_height == 0 {
        return Err(RookLWError::Image(format!(
            "Image {}x{} is too small for {} divisions",
            a.width, a.height, divisions
        )));
    }

    let mut differences = Vec::with_capacity(divisions * divisions);

    for box_y in 0..divisions {
        for box_x in 0..divisions {
            let start_x = box_x * box_width;
            let start_y = box_y * box_height;
            let end_x = if box_x == divisions - 1 {
                a.width
            } else {
                (box_x + 1) * box_width
            };
            let end_y = if box_y == divisions - 1 {
                a.height
            } else {
                (box_y + 1) * box_height
            };

            let mut sum_a: u64 = 0;
            let mut sum_b: u64 = 0;
            let mut pixel_count: u64 = 0;

            for y in start_y..end_y {
                let a_row = y
                    .checked_mul(a.stride)
                    .ok_or_else(|| RookLWError::Image("YPlane index overflow".to_string()))?;
                let b_row = y
                    .checked_mul(b.stride)
                    .ok_or_else(|| RookLWError::Image("YPlane index overflow".to_string()))?;

                for x in start_x..end_x {
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

                    sum_a += av as u64;
                    sum_b += bv as u64;
                    pixel_count += 1;
                }
            }

            if pixel_count == 0 {
                return Err(RookLWError::Image(
                    "Box has zero pixels".to_string(),
                ));
            }

            let avg_a = sum_a as f32 / pixel_count as f32;
            let avg_b = sum_b as f32 / pixel_count as f32;
            let diff = (avg_a - avg_b).abs() / 255.0;

            differences.push(diff);
        }
    }

    Ok(differences)
}

