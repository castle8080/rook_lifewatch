use crate::{RookLWResult, RookLWError};
use crate::image::frame::Frame;
use crate::image::fourcc::FOURCC_BGR3;
use opencv::prelude::*;

/// A frame captured from an OpenCV VideoCapture source.
///
/// OpenCV typically stores images as BGR-formatted Mats, which are single-plane
/// interleaved pixel data. This struct wraps an owned `Mat` and implements the
/// `Frame` trait to provide uniform access to the underlying image data.
pub struct OpenCvFrame {
    mat: opencv::core::Mat,
}

impl OpenCvFrame {
    /// Create a new OpenCvFrame from an OpenCV Mat.
    ///
    /// The Mat should contain valid image data (not be empty).
    pub fn new(mat: opencv::core::Mat) -> RookLWResult<Self> {
        if mat.empty() {
            return Err(RookLWError::Image("Cannot create frame from empty Mat".to_string()));
        }
        Ok(Self { mat })
    }

    /// Get a reference to the underlying OpenCV Mat.
    pub fn mat(&self) -> &opencv::core::Mat {
        &self.mat
    }
}

impl Frame for OpenCvFrame {
    fn get_plane_count(&self) -> RookLWResult<usize> {
        // OpenCV Mat is typically a single contiguous plane (interleaved channels)
        Ok(1)
    }

    fn get_plane_data(&self, plane_index: usize) -> RookLWResult<&[u8]> {
        if plane_index != 0 {
            return Err(RookLWError::Image(
                format!("Invalid plane index {} for OpenCV frame (only plane 0 exists)", plane_index),
            ));
        }

        // Ensure the Mat is continuous in memory
        if !self.mat.is_continuous() {
            return Err(RookLWError::Image(
                "OpenCV Mat is not continuous in memory".to_string(),
            ));
        }

        let data = self.mat.data_bytes().map_err(|e| {
            RookLWError::Image(format!("Failed to get frame data: {}", e))
        })?;

        Ok(data)
    }

    fn get_pixel_format(&self) -> RookLWResult<u32> {
        // OpenCV VideoCapture typically returns BGR24 format by default.
        // We use the BGR3 FourCC to represent this.
        Ok(FOURCC_BGR3)
    }

    fn get_width(&self) -> RookLWResult<usize> {
        let cols = self.mat.cols();
        if cols <= 0 {
            return Err(RookLWError::Image("Frame has invalid width".to_string()));
        }
        Ok(cols as usize)
    }

    fn get_height(&self) -> RookLWResult<usize> {
        let rows = self.mat.rows();
        if rows <= 0 {
            return Err(RookLWError::Image("Frame has invalid height".to_string()));
        }
        Ok(rows as usize)
    }
}
