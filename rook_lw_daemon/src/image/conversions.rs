use std::borrow::Cow;
use std::io::Cursor;

use crate::events::capture_event::CaptureEvent;
use crate::image::fourcc;
use crate::image::frame::{FrameError, FrameResult};

/// Convert a `CaptureEvent` into JPEG bytes without forcing a copy.
///
/// - MJPG: returns `Cow::Borrowed(&event.image_data[0])`
/// - Other formats: returns `Cow::Owned(Vec<u8>)` containing a freshly encoded JPEG
pub fn capture_event_to_jpeg(event: &CaptureEvent) -> FrameResult<Cow<'_, [u8]>> {
    capture_event_to_jpeg_with_quality(event, 85)
}

/// Same as `capture_event_to_jpeg`, but allows specifying JPEG quality (1-100).
pub fn capture_event_to_jpeg_with_quality(
    event: &CaptureEvent,
    quality: u8,
) -> FrameResult<Cow<'_, [u8]>> {
    validate_has_planes(event)?;
    
    let pixel_format = event.pixel_format;

    if pixel_format == fourcc::FOURCC_MJPG {
        return mjpg_event_to_jpeg(event);
    }

    let rgb = if pixel_format == fourcc::FOURCC_YUYV {
        rgb_from_yuyv_event(event)?
    } else if pixel_format == fourcc::FOURCC_NV12 {
        rgb_from_nv12_event(event)?
    } else if pixel_format == fourcc::FOURCC_YU12 {
        rgb_from_yu12_event(event)?
    } else if pixel_format == fourcc::FOURCC_RGB3 {
        rgb_from_rgb3_event(event)?
    } else if pixel_format == fourcc::FOURCC_BGR3 {
        rgb_from_bgr3_event(event)?
    } else {
        return Err(FrameError::ProcessingError(format!(
            "Unsupported pixel format for JPEG conversion: {}",
            fourcc::fourcc_to_string(pixel_format)
        )));
    };

    Ok(Cow::Owned(encode_rgb_to_jpeg(
        event.width,
        event.height,
        &rgb,
        quality,
    )?))
}

fn validate_has_planes(event: &CaptureEvent) -> FrameResult<()> {
    if event.image_data.is_empty() {
        return Err(FrameError::ProcessingError(
            "CaptureEvent has no planes".to_string(),
        ));
    }
    Ok(())
}

fn mjpg_event_to_jpeg(event: &CaptureEvent) -> FrameResult<Cow<'_, [u8]>> {
    validate_has_planes(event)?;
    Ok(Cow::Borrowed(&event.image_data[0]))
}

fn rgb_from_yuyv_event(event: &CaptureEvent) -> FrameResult<Vec<u8>> {
    let width = event.width;
    let height = event.height;

    if width % 2 != 0 {
        return Err(FrameError::ProcessingError(format!(
            "YUYV conversion requires even width, got {width}"
        )));
    }

    let yuyv = &event.image_data[0];
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

    yuyv_to_rgb_interleaved(width, height, &yuyv[..expected_len])
}

fn rgb_from_nv12_event(event: &CaptureEvent) -> FrameResult<Vec<u8>> {
    let width = event.width;
    let height = event.height;

    if width % 2 != 0 || height % 2 != 0 {
        return Err(FrameError::ProcessingError(format!(
            "NV12 conversion requires even width/height, got {width}x{height}"
        )));
    }
    if event.image_data.len() < 2 {
        return Err(FrameError::ProcessingError(format!(
            "NV12 requires 2 planes, got {}",
            event.image_data.len()
        )));
    }

    let y = &event.image_data[0];
    let uv = &event.image_data[1];
    let expected_y = width
        .checked_mul(height)
        .ok_or_else(|| FrameError::ProcessingError("Frame dimensions overflow".to_string()))?;
    let expected_uv = expected_y
        .checked_div(2)
        .ok_or_else(|| FrameError::ProcessingError("Frame dimensions overflow".to_string()))?;

    if y.len() < expected_y {
        return Err(FrameError::ProcessingError(format!(
            "NV12 Y plane too small: got {}, expected at least {}",
            y.len(), expected_y
        )));
    }
    if uv.len() < expected_uv {
        return Err(FrameError::ProcessingError(format!(
            "NV12 UV plane too small: got {}, expected at least {}",
            uv.len(), expected_uv
        )));
    }

    nv12_to_rgb_interleaved(width, height, &y[..expected_y], &uv[..expected_uv])
}

fn rgb_from_yu12_event(event: &CaptureEvent) -> FrameResult<Vec<u8>> {
    let width = event.width;
    let height = event.height;

    if width % 2 != 0 || height % 2 != 0 {
        return Err(FrameError::ProcessingError(format!(
            "YU12/I420 conversion requires even width/height, got {width}x{height}"
        )));
    }
    if event.image_data.len() < 3 {
        return Err(FrameError::ProcessingError(format!(
            "YU12/I420 requires 3 planes, got {}",
            event.image_data.len()
        )));
    }

    let y = &event.image_data[0];
    let u = &event.image_data[1];
    let v = &event.image_data[2];

    let expected_y = width
        .checked_mul(height)
        .ok_or_else(|| FrameError::ProcessingError("Frame dimensions overflow".to_string()))?;
    let expected_uv = expected_y
        .checked_div(4)
        .ok_or_else(|| FrameError::ProcessingError("Frame dimensions overflow".to_string()))?;

    if y.len() < expected_y {
        return Err(FrameError::ProcessingError(format!(
            "YU12 Y plane too small: got {}, expected at least {}",
            y.len(), expected_y
        )));
    }
    if u.len() < expected_uv {
        return Err(FrameError::ProcessingError(format!(
            "YU12 U plane too small: got {}, expected at least {}",
            u.len(), expected_uv
        )));
    }
    if v.len() < expected_uv {
        return Err(FrameError::ProcessingError(format!(
            "YU12 V plane too small: got {}, expected at least {}",
            v.len(), expected_uv
        )));
    }

    i420_to_rgb_interleaved(
        width,
        height,
        &y[..expected_y],
        &u[..expected_uv],
        &v[..expected_uv],
    )
}

fn rgb_from_rgb3_event(event: &CaptureEvent) -> FrameResult<Vec<u8>> {
    let width = event.width;
    let height = event.height;

    let expected = width
        .checked_mul(height)
        .and_then(|px| px.checked_mul(3))
        .ok_or_else(|| FrameError::ProcessingError("Frame dimensions overflow".to_string()))?;

    let src = &event.image_data[0];
    if src.len() < expected {
        return Err(FrameError::ProcessingError(format!(
            "RGB3 buffer too small: got {}, expected at least {}",
            src.len(), expected
        )));
    }

    Ok(src[..expected].to_vec())
}

fn rgb_from_bgr3_event(event: &CaptureEvent) -> FrameResult<Vec<u8>> {
    let width = event.width;
    let height = event.height;

    let expected = width
        .checked_mul(height)
        .and_then(|px| px.checked_mul(3))
        .ok_or_else(|| FrameError::ProcessingError("Frame dimensions overflow".to_string()))?;

    let src = &event.image_data[0];
    if src.len() < expected {
        return Err(FrameError::ProcessingError(format!(
            "BGR3 buffer too small: got {}, expected at least {}",
            src.len(), expected
        )));
    }

    let mut out = vec![0u8; expected];
    for i in (0..expected).step_by(3) {
        out[i] = src[i + 2];
        out[i + 1] = src[i + 1];
        out[i + 2] = src[i];
    }
    Ok(out)
}

fn encode_rgb_to_jpeg(width: usize, height: usize, rgb: &[u8], quality: u8) -> FrameResult<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
            Cursor::new(&mut out),
            quality.clamp(1, 100),
        );

        encoder
            .encode(
                rgb,
                width as u32,
                height as u32,
                image::ColorType::Rgb8.into(),
            )
            .map_err(|e| FrameError::ProcessingError(format!("JPEG encode error: {e}")))?;
    }
    Ok(out)
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

fn nv12_to_rgb_interleaved(
    width: usize,
    height: usize,
    y: &[u8],
    uv: &[u8],
) -> FrameResult<Vec<u8>> {
    let pixel_count = width
        .checked_mul(height)
        .ok_or_else(|| FrameError::ProcessingError("Frame dimensions overflow".to_string()))?;
    let mut rgb = vec![0u8; pixel_count * 3];

    let uv_width = width;
    for row in 0..height {
        let y_row = row * width;
        let uv_row = (row / 2) * uv_width;
        for col in 0..width {
            let yv = y[y_row + col] as i32;
            let uv_index = uv_row + (col / 2) * 2;
            let u = uv[uv_index] as i32;
            let v = uv[uv_index + 1] as i32;

            let (r, g, b) = yuv_to_rgb(yv, u, v);
            let dst = (y_row + col) * 3;
            rgb[dst] = r;
            rgb[dst + 1] = g;
            rgb[dst + 2] = b;
        }
    }

    Ok(rgb)
}

fn i420_to_rgb_interleaved(
    width: usize,
    height: usize,
    y: &[u8],
    u: &[u8],
    v: &[u8],
) -> FrameResult<Vec<u8>> {
    let pixel_count = width
        .checked_mul(height)
        .ok_or_else(|| FrameError::ProcessingError("Frame dimensions overflow".to_string()))?;
    let mut rgb = vec![0u8; pixel_count * 3];

    let chroma_width = width / 2;
    for row in 0..height {
        let y_row = row * width;
        let c_row = (row / 2) * chroma_width;
        for col in 0..width {
            let yv = y[y_row + col] as i32;
            let c_index = c_row + (col / 2);
            let uv = u[c_index] as i32;
            let vv = v[c_index] as i32;

            let (r, g, b) = yuv_to_rgb(yv, uv, vv);
            let dst = (y_row + col) * 3;
            rgb[dst] = r;
            rgb[dst + 1] = g;
            rgb[dst + 2] = b;
        }
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
