use crate::core::frame::Frame;
use crate::core::motion::MotionEvent;

#[derive(Debug, thiserror::Error)]
pub enum ProcessingError {
    #[error("processing error: {0}")]
    Generic(String),
}

pub trait FrameProcessor: Send {
    fn process_frame(&mut self, frame: Box<dyn Frame>) -> Result<Vec<MotionEvent>, ProcessingError>;
}

pub trait EventSink: Send {
    fn handle_event(&mut self, event: MotionEvent);
}
