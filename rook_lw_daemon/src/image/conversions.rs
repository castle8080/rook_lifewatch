use std::io::Cursor;

use crate::image::fourcc;
use crate::image::frame::{Frame, FrameError, FrameResult};

/// Convert a `Frame` into an owned JPEG byte buffer.
///
/// - If the source is MJPG, this clones the JPEG bitstream (fast path).
/// - If the source is YUYV, this converts to RGB and JPEG-encodes it.
///
/// For other formats, returns an error.
pub fn frame_to_jpeg_bytes(frame: &dyn Frame) -> FrameResult<Vec<u8>> {
    frame_to_jpeg_bytes_with_quality(frame, 85)
}

/// Same as `frame_to_jpeg_bytes`, but allows specifying JPEG quality (1-100).
pub fn frame_to_jpeg_bytes_with_quality(frame: &dyn Frame, quality: u8) -> FrameResult<Vec<u8>> {
    if frame.get_plane_count()? < 1 {
        return Err(FrameError::ProcessingError("Frame has no planes".to_string()));
    }

    let pixel_format = frame.get_pixel_format()?;

    if pixel_format == fourcc::FOURCC_MJPG {
        // MJPG frames are typically already JPEG bitstreams.
        return Ok(frame.get_plane_data(0)?.to_vec());
    }

    if pixel_format == fourcc::FOURCC_YUYV {
        let width = frame.get_width()?;
        let height = frame.get_height()?;

        if width % 2 != 0 {
            return Err(FrameError::ProcessingError(format!(
                "YUYV conversion requires even width, got {width}" 
            )));
        }

        let yuyv = frame.get_plane_data(0)?;
        let expected_len = width
            .checked_mul(height)
            .and_then(|px| px.checked_mul(2))
            .ok_or_else(|| FrameError::ProcessingError("Frame dimensions overflow".to_string()))?;

        if yuyv.len() < expected_len {
            return Err(FrameError::ProcessingError(format!(
                "YUYV buffer too small: got {}, expected at least {}",
                yuyv.len(), expected_len
            )));
        }

        let rgb = yuyv_to_rgb_interleaved(width, height, &yuyv[..expected_len])?;

        let mut out = Vec::new();
        {
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                Cursor::new(&mut out),
                quality.clamp(1, 100),
            );

            encoder
                .encode(
                    &rgb,
                    width as u32,
                    height as u32,
                    image::ColorType::Rgb8.into(),
                )
                .map_err(|e| FrameError::ProcessingError(format!("JPEG encode error: {e}")))?;
        }

        return Ok(out);
    }

    Err(FrameError::ProcessingError(format!(
        "Unsupported pixel format for JPEG conversion: {}",
        fourcc::fourcc_to_string(pixel_format)
    )))
}

fn yuyv_to_rgb_interleaved(width: usize, height: usize, yuyv: &[u8]) -> FrameResult<Vec<u8>> {
    let pixel_count = width
        .checked_mul(height)
        .ok_or_else(|| FrameError::ProcessingError("Frame dimensions overflow".to_string()))?;
    let mut rgb = vec![0u8; pixel_count * 3];

    // YUYV packed: [Y0 U Y1 V] per 2 pixels.
    // Standard integer conversion (BT.601-ish), clamped to 0..255.
    let mut src = 0usize;
    let mut dst = 0usize;

    while src + 3 < yuyv.len() {
        let y0 = yuyv[src] as i32;
        let u = yuyv[src + 1] as i32;
        let y1 = yuyv[src + 2] as i32;
        let v = yuyv[src + 3] as i32;

        let (r0, g0, b0) = yuv_to_rgb(y0, u, v);
        let (r1, g1, b1) = yuv_to_rgb(y1, u, v);

        if dst + 5 >= rgb.len() {
            break;
        }

        rgb[dst] = r0;
        rgb[dst + 1] = g0;
        rgb[dst + 2] = b0;
        rgb[dst + 3] = r1;
        rgb[dst + 4] = g1;
        rgb[dst + 5] = b1;

        src += 4;
        dst += 6;
    }

    Ok(rgb)
}

#[inline]
fn yuv_to_rgb(y: i32, u: i32, v: i32) -> (u8, u8, u8) {
    // ITU-R BT.601 conversion (common for camera YUV), using integer math.
    // C = Y - 16, D = U - 128, E = V - 128
    let c = y - 16;
    let d = u - 128;
    let e = v - 128;

    let r = (298 * c + 409 * e + 128) >> 8;
    let g = (298 * c - 100 * d - 208 * e + 128) >> 8;
    let b = (298 * c + 516 * d + 128) >> 8;

    (clamp_u8(r), clamp_u8(g), clamp_u8(b))
}

#[inline]
fn clamp_u8(v: i32) -> u8 {
    v.clamp(0, 255) as u8
}
