use std::borrow::Cow;
use std::io::Cursor;

use crate::{RookLWError, RookLWResult};

use crate::image::frame::Frame;
use crate::image::fourcc::{FOURCC_MJPG, FOURCC_YUYV, FOURCC_NV12, FOURCC_YU12, FOURCC_RGB3, FOURCC_BGR3, fourcc_to_string};

use image::{DynamicImage, RgbImage, GenericImageView};

/// Encode a DynamicImage to JPEG bytes. Default quality is 85 if not specified.
pub fn dynamic_image_to_jpeg(img: &DynamicImage, quality: Option<u8>) -> RookLWResult<Vec<u8>> {
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();
    let mut out = Vec::new();
    let q = quality.unwrap_or(85).clamp(1, 100);
    {
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(std::io::Cursor::new(&mut out), q);
        encoder.encode(&rgb, width, height, image::ColorType::Rgb8.into())?;
    }
    Ok(out)
}

/// Convert a Frame into a DynamicImage, supporting various pixel formats.
pub fn frame_to_dynamic_image<F: Frame + ?Sized>(frame: &F) -> RookLWResult<DynamicImage> {
    let pixel_format = frame.get_pixel_format()?;

    let width = frame.get_width()?;
    let height = frame.get_height()?;

    let rgb: Vec<u8> = if pixel_format == FOURCC_RGB3 {
        rgb_from_rgb3_view(frame)?
    } else if pixel_format == FOURCC_BGR3 {
        rgb_from_bgr3_view(frame)?
    } else if pixel_format == FOURCC_YUYV {
        rgb_from_yuyv_view(frame)?
    } else if pixel_format == FOURCC_NV12 {
        rgb_from_nv12_view(frame)?
    } else if pixel_format == FOURCC_YU12 {
        rgb_from_yu12_view(frame)?
    } else if pixel_format == FOURCC_MJPG {
        // Decode JPEG to DynamicImage directly
        let jpeg = mjpg_plane_to_jpeg(frame)?;
        let img = image::load_from_memory_with_format(&jpeg, image::ImageFormat::Jpeg)
            .map_err(|e| RookLWError::Image(format!("JPEG decode error: {e}")))?;
        return Ok(img);
    } else {
        return Err(RookLWError::Image(format!(
            "Unsupported pixel format for DynamicImage conversion: {}",
            fourcc_to_string(pixel_format)
        )));
    };
    // Construct RgbImage from raw RGB buffer
    let img = RgbImage::from_raw(width as u32, height as u32, rgb)
        .ok_or_else(|| RookLWError::Image("Failed to create RgbImage from buffer".to_string()))?;
    Ok(DynamicImage::ImageRgb8(img))
}

fn mjpg_plane_to_jpeg<F: Frame + ?Sized>(frame: &F) -> RookLWResult<Cow<'_, [u8]>> {
    if frame.get_plane_count()? == 0 {
        return Err(RookLWError::Image("Frame has no planes".into()));
    }
    let data = frame.get_plane_data(0).map_err(|e| RookLWError::Image(format!("Missing MJPG plane: {e}")))?;
    Ok(Cow::Borrowed(&data))
}
    
// Generic FrameView-based helpers for conversion
fn rgb_from_rgb3_view<F: Frame + ?Sized>(frame: &F) -> RookLWResult<Vec<u8>> {
    let width = frame.get_width()?;
    let height = frame.get_height()?;
    let expected = width.checked_mul(height).and_then(|px| px.checked_mul(3)).ok_or_else(|| RookLWError::Image("Frame dimensions overflow".to_string()))?;
    let src = frame.get_plane_data(0).map_err(|e| RookLWError::Image(format!("Missing RGB3 plane: {e}")))?;
    if src.len() < expected {
        return Err(RookLWError::Image(format!("RGB3 buffer too small: got {}, expected at least {}", src.len(), expected)));
    }
    Ok(src[..expected].to_vec())
}

fn rgb_from_bgr3_view<F: Frame + ?Sized>(frame: &F) -> RookLWResult<Vec<u8>> {
    let width = frame.get_width()?;
    let height = frame.get_height()?;
    let expected = width.checked_mul(height).and_then(|px| px.checked_mul(3)).ok_or_else(|| RookLWError::Image("Frame dimensions overflow".to_string()))?;
    let src = frame.get_plane_data(0).map_err(|e| RookLWError::Image(format!("Missing BGR3 plane: {e}")))?;
    if src.len() < expected {
        return Err(RookLWError::Image(format!("BGR3 buffer too small: got {}, expected at least {}", src.len(), expected)));
    }
    let mut out = vec![0u8; expected];
    for i in (0..expected).step_by(3) {
        out[i] = src[i + 2];
        out[i + 1] = src[i + 1];
        out[i + 2] = src[i];
    }
    Ok(out)
}

fn rgb_from_yuyv_view<F: Frame + ?Sized>(frame: &F) -> RookLWResult<Vec<u8>> {
    let width = frame.get_width()?;
    let height = frame.get_height()?;
    if width % 2 != 0 {
        return Err(RookLWError::Image(format!("YUYV conversion requires even width, got {width}")));
    }
    let yuyv = frame.get_plane_data(0).map_err(|e| RookLWError::Image(format!("Missing YUYV plane: {e}")))?;
    let expected_len = width.checked_mul(height).and_then(|px| px.checked_mul(2)).ok_or_else(|| RookLWError::Image("Frame dimensions overflow".to_string()))?;
    if yuyv.len() < expected_len {
        return Err(RookLWError::Image(format!("YUYV buffer too small: got {}, expected at least {}", yuyv.len(), expected_len)));
    }
    yuyv_to_rgb_interleaved(width, height, &yuyv[..expected_len])
}

fn rgb_from_nv12_view<F: Frame + ?Sized>(frame: &F) -> RookLWResult<Vec<u8>> {
    let width = frame.get_width()?;
    let height = frame.get_height()?;
    if width % 2 != 0 || height % 2 != 0 {
        return Err(RookLWError::Image(format!("NV12 conversion requires even width/height, got {width}x{height}")));
    }
    if frame.get_plane_count()? < 2 {
        return Err(RookLWError::Image(format!("NV12 requires 2 planes, got {}", frame.get_plane_count()?)));
    }
    let y = frame.get_plane_data(0).map_err(|e| RookLWError::Image(format!("Missing NV12 Y plane: {e}")))?;
    let uv = frame.get_plane_data(1).map_err(|e| RookLWError::Image(format!("Missing NV12 UV plane: {e}")))?;
    let expected_y = width.checked_mul(height).ok_or_else(|| RookLWError::Image("Frame dimensions overflow".to_string()))?;
    let expected_uv = expected_y.checked_div(2).ok_or_else(|| RookLWError::Image("Frame dimensions overflow".to_string()))?;
    if y.len() < expected_y {
        return Err(RookLWError::Image(format!("NV12 Y plane too small: got {}, expected at least {}", y.len(), expected_y)));
    }
    if uv.len() < expected_uv {
        return Err(RookLWError::Image(format!("NV12 UV plane too small: got {}, expected at least {}", uv.len(), expected_uv)));
    }
    nv12_to_rgb_interleaved(width, height, &y[..expected_y], &uv[..expected_uv])
}

fn rgb_from_yu12_view<F: Frame + ?Sized>(frame: &F) -> RookLWResult<Vec<u8>> {
    let width = frame.get_width()?;
    let height = frame.get_height()?;
    if width % 2 != 0 || height % 2 != 0 {
        return Err(RookLWError::Image(format!("YU12/I420 conversion requires even width/height, got {width}x{height}")));
    }
    if frame.get_plane_count()? < 3 {
        return Err(RookLWError::Image(format!("YU12/I420 requires 3 planes, got {}", frame.get_plane_count()?)));
    }

    let y = frame.get_plane_data(0).map_err(|e| RookLWError::Image(format!("Missing YU12 Y plane: {e}")))?;
    let u = frame.get_plane_data(1).map_err(|e| RookLWError::Image(format!("Missing YU12 U plane: {e}")))?;
    let v = frame.get_plane_data(2).map_err(|e| RookLWError::Image(format!("Missing YU12 V plane: {e}")))?;

    let expected_y = width.checked_mul(height).ok_or_else(|| RookLWError::Image("Frame dimensions overflow".to_string()))?;
    let expected_uv = expected_y.checked_div(4).ok_or_else(|| RookLWError::Image("Frame dimensions overflow".to_string()))?;
    if y.len() < expected_y {
        return Err(RookLWError::Image(format!("YU12 Y plane too small: got {}, expected at least {}", y.len(), expected_y)));
    }
    if u.len() < expected_uv {
        return Err(RookLWError::Image(format!("YU12 U plane too small: got {}, expected at least {}", u.len(), expected_uv)));
    }
    if v.len() < expected_uv {
        return Err(RookLWError::Image(format!("YU12 V plane too small: got {}, expected at least {}", v.len(), expected_uv)));
    }
    i420_to_rgb_interleaved(width, height, &y[..expected_y], &u[..expected_uv], &v[..expected_uv])
}

/// Decode JPEG bytes and re-encode them at a (typically lower) JPEG quality.
///
/// This is useful when the camera outputs MJPEG at very high quality (large files)
/// and you want smaller JPEGs.
pub fn reencode_jpeg(jpeg_data: &[u8]) -> RookLWResult<Vec<u8>> {
    reencode_jpeg_with_quality(jpeg_data, 85)
}

/// Same as `reencode_jpeg` but allows specifying JPEG quality (1-100).
pub fn reencode_jpeg_with_quality(jpeg_data: &[u8], quality: u8) -> RookLWResult<Vec<u8>> {
    let decoded = image::load_from_memory_with_format(jpeg_data, image::ImageFormat::Jpeg)?;
    let (width, height) = decoded.dimensions();
    let rgb = decoded.to_rgb8().into_raw();
    encode_rgb_to_jpeg(width as usize, height as usize, &rgb, quality)
}

fn encode_rgb_to_jpeg(width: usize, height: usize, rgb: &[u8], quality: u8) -> RookLWResult<Vec<u8>> {
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
            .map_err(|e| RookLWError::Image(format!("JPEG encode error: {e}")))?;
    }
    Ok(out)
}

fn yuyv_to_rgb_interleaved(width: usize, height: usize, yuyv: &[u8]) -> RookLWResult<Vec<u8>> {
    let pixel_count = width
        .checked_mul(height)
        .ok_or_else(|| RookLWError::Image("Frame dimensions overflow".to_string()))?;
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
) -> RookLWResult<Vec<u8>> {
    let pixel_count = width
        .checked_mul(height)
        .ok_or_else(|| RookLWError::Image("Frame dimensions overflow".to_string()))?;
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
) -> RookLWResult<Vec<u8>> {
    let pixel_count = width
        .checked_mul(height)
        .ok_or_else(|| RookLWError::Image("Frame dimensions overflow".to_string()))?;
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
