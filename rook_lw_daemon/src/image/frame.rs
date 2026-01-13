
#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("capture error: {0}")]
    Capture(String),
    #[error("no frame source implementation available")]
    NoImplementationAvailable,
    #[error("failed to initialize frame source: {0}")]
    InitializationFailed(String),
    #[error("processing error: {0}")]
    ProcessingError(String),
}

pub type FrameResult<T> = Result<T, FrameError>;

impl From<image::ImageError> for FrameError {
    fn from(err: image::ImageError) -> Self {
        FrameError::ProcessingError(format!("image error: {err}"))
    }
}

impl From<std::io::Error> for FrameError {
    fn from(err: std::io::Error) -> Self {
        FrameError::ProcessingError(format!("io error: {err}"))
    }
}

impl From<anyhow::Error> for FrameError {
    fn from(err: anyhow::Error) -> Self {
        FrameError::ProcessingError(format!("anyhow error: {err}"))
    }
}

impl From<opencv::Error> for FrameError {
    fn from(err: opencv::Error) -> Self {
        FrameError::ProcessingError(format!("opencv error: {err}"))
    }
}

impl From<serde_json::Error> for FrameError {
    fn from(err: serde_json::Error) -> Self {
        FrameError::ProcessingError(format!("json error: {err}"))
    }
}

pub trait Frame {
    fn get_plane_count(&self) -> FrameResult<usize>;
    fn get_plane_data(&self, plane_index: usize) -> FrameResult<&[u8]>;
    fn get_pixel_format(&self) -> FrameResult<u32>;
    fn get_width(&self) -> FrameResult<usize>;
    fn get_height(&self) -> FrameResult<usize>;
}

pub trait FrameSource {

    fn list_sources(&mut self) -> FrameResult<Vec<String>>;

    fn set_source(&mut self, source: &str) -> FrameResult<()>;

    fn start(&mut self) -> FrameResult<()>;

    fn stop(&mut self) -> FrameResult<()>;

    /// Returns the next frame.
    ///
    /// The returned frame is constrained to live no longer than the borrow of
    /// this `FrameSource` reference (i.e. it cannot outlive the `FrameSource`
    /// instance it came from).
    ///
    /// `&self` (shared borrow) is used specifically so you can hold multiple
    /// frames at once and still acquire subsequent frames. Implementations that
    /// need to mutate internal state should use interior mutability.
    fn next_frame(&self) -> FrameResult<Box<dyn Frame + '_>>;

    fn get_pixel_format(&self) -> FrameResult<u32>;
    fn get_width(&self) -> FrameResult<usize>;
    fn get_height(&self) -> FrameResult<usize>;
}
