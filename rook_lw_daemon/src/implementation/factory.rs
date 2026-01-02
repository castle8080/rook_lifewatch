use crate::core::frame::{FrameError, FrameResult, FrameSource};

pub struct FrameSourceFactory;

impl FrameSourceFactory {

    /// Create a frame source using the default preference order
    pub fn create() -> FrameResult<Box<dyn FrameSource>> {
        Self::available_sources().first()
            .ok_or(FrameError::NoImplementationAvailable)
            .and_then(|source_name| Self::try_create(source_name))
    }

    /// Try to create a specific frame source by name
    fn try_create(source_name: &str) -> FrameResult<Box<dyn FrameSource>> {
        match source_name {
            "libcamera" => {
                try_create_libcamera_source()
            }
            "opencv" => {
                try_create_opencv_source()
            }
            _ => Err(FrameError::InitializationFailed(format!(
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


fn try_create_libcamera_source() -> FrameResult<Box<dyn FrameSource>> {
    #[cfg(feature = "libcamera")]
    {
        use crate::implementation::libcamera::LibCameraFrameSource;
        return LibCameraFrameSource::try_new()
            .map(|src| Box::new(src) as Box<dyn FrameSource>)
            .map_err(|e| FrameError::InitializationFailed(e.to_string()));
    }
    #[cfg(not(feature = "libcamera"))]
    {
        Err(FrameError::InitializationFailed(
            "libcamera feature not enabled".to_string(),
        ))
    }
}

fn try_create_opencv_source() -> FrameResult<Box<dyn FrameSource>> {
    #[cfg(feature = "opencv")]
    {
        use crate::implementation::opencv::OpencvFrameSource;
        return OpencvFrameSource::try_new()
            .map(|src| Box::new(src) as Box<dyn FrameSource>)
            .map_err(|e| FrameError::InitializationFailed(e.to_string()));
    }
    #[cfg(not(feature = "opencv"))]
    {
        Err(FrameError::InitializationFailed(
            "opencv feature not enabled".to_string(),
        ))
    }
}
