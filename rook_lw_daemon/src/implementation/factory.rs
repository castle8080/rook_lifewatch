use crate::core::frame::FrameSource;

#[derive(Debug, thiserror::Error)]
pub enum FactoryError {
    #[error("no frame source implementation available")]
    NoImplementationAvailable,
    #[error("failed to initialize frame source: {0}")]
    InitializationFailed(String),
}

/// Priority order for frame source selection at runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameSourcePreference {
    /// Try libcamera first, then opencv
    PreferLibCamera,
    /// Try opencv first, then libcamera
    PreferOpenCV,
}

impl Default for FrameSourcePreference {
    fn default() -> Self {
        Self::PreferLibCamera
    }
}

/// Factory for creating frame sources based on compile-time features
/// and runtime availability
pub struct FrameSourceFactory;

impl FrameSourceFactory {
    /// Create a frame source using the default preference order
    pub fn create() -> Result<Box<dyn FrameSource>, FactoryError> {
        Self::create_with_preference(FrameSourcePreference::default())
    }

    /// Create a frame source using the specified preference order
    pub fn create_with_preference(
        preference: FrameSourcePreference,
    ) -> Result<Box<dyn FrameSource>, FactoryError> {
        let sources = match preference {
            FrameSourcePreference::PreferLibCamera => vec!["libcamera", "opencv"],
            FrameSourcePreference::PreferOpenCV => vec!["opencv", "libcamera"],
        };

        for source_name in sources {
            match Self::try_create(source_name) {
                Ok(source) => {
                    println!("Successfully initialized {} frame source", source_name);
                    return Ok(source);
                }
                Err(e) => {
                    println!("Failed to initialize {} frame source: {}", source_name, e);
                    continue;
                }
            }
        }

        Err(FactoryError::NoImplementationAvailable)
    }

    /// Try to create a specific frame source by name
    fn try_create(source_name: &str) -> Result<Box<dyn FrameSource>, FactoryError> {
        match source_name {
            #[cfg(feature = "libcamera")]
            "libcamera" => {
                use crate::implementation::libcamera::LibCameraFrameSource;
                LibCameraFrameSource::try_new()
                    .map(|src| Box::new(src) as Box<dyn FrameSource>)
                    .map_err(|e| FactoryError::InitializationFailed(e.to_string()))
            }
            #[cfg(feature = "opencv")]
            "opencv" => {
                use crate::implementation::opencv::OpencvFrameSource;
                OpencvFrameSource::try_new()
                    .map(|src| Box::new(src) as Box<dyn FrameSource>)
                    .map_err(|e| FactoryError::InitializationFailed(e.to_string()))
            }
            _ => Err(FactoryError::InitializationFailed(format!(
                "unknown or disabled source: {}",
                source_name
            ))),
        }
    }

    /// Get a list of frame sources compiled into this binary
    pub fn available_sources() -> Vec<&'static str> {
        let mut sources = Vec::new();
        #[cfg(feature = "libcamera")]
        sources.push("libcamera");
        #[cfg(feature = "opencv")]
        sources.push("opencv");
        sources
    }
}
