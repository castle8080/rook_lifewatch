pub trait Frame {
}

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

pub trait FrameSource {
    fn start(&mut self) -> FrameResult<()>;

    fn stop(&mut self) -> FrameResult<()>;

    fn next_frame(&mut self) -> FrameResult<Box<dyn Frame>>;

    fn list_sources(&mut self) -> FrameResult<Vec<String>>;

    fn set_source(&mut self, source: &str) -> FrameResult<()>;
}
