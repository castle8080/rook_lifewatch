use super::FrameSource;
use crate::{RookLWResult, RookLWError};

pub struct FrameSourceFactory;

impl FrameSourceFactory {

    /// Create a frame source using the default preference order
    pub fn create() -> RookLWResult<Box<dyn FrameSource + Send + Sync>> {
        let sources = Self::available_sources();
        let source_name = sources
            .first()
            .ok_or(RookLWError::Initialization("No available implementations of a FrameSource.".into()))?;

        Self::try_create(source_name)
    }

    /// Try to create a specific frame source by name
    pub fn try_create(source_name: &str) -> RookLWResult<Box<dyn FrameSource + Send + Sync>> {
        match source_name {
            "libcamera" => {
                try_create_libcamera_source()
            }
            "opencv" => {
                try_create_opencv_source()
            }
            _ => Err(RookLWError::Initialization(format!(
                "unknown or disabled source: {}",
                source_name
            ))),
        }
    }

    /// Get a list of frame sources compiled into this binary
    pub fn available_sources() -> Vec<&'static str> {
        vec![
            #[cfg(feature = "libcamera")]
            "libcamera",
            #[cfg(feature = "opencv")]
            "opencv",
        ]
    }
}

fn try_create_libcamera_source() -> RookLWResult<Box<dyn FrameSource + Send + Sync>> {
    #[cfg(feature = "libcamera")]
    {
        use crate::image::libcamera::LibCameraFrameSource;
        return Ok(Box::new(LibCameraFrameSource::new()?));
    }
    #[cfg(not(feature = "libcamera"))]
    {
        Err(RookLWError::Initialization(
            "libcamera feature not enabled".to_string(),
        ))
    }
}

fn try_create_opencv_source() -> RookLWResult<Box<dyn FrameSource + Send + Sync>> {
    #[cfg(feature = "opencv")]
    {
        use crate::image::opencv::OpenCvFrameSource;
        return Ok(Box::new(OpenCvFrameSource::new()?));
    }
    #[cfg(not(feature = "opencv"))]
    {
        Err(RookLWError::Initialization(
            "opencv feature not enabled".to_string(),
        ))
    }
}
