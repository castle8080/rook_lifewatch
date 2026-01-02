use crate::core::frame::Frame;
use crate::core::motion::{MotionEvent, MotionEventKind};
use crate::core::pipeline::{FrameProcessor, ProcessingError};
use std::time::SystemTime;

pub struct SimpleFrameDiffer;

impl FrameProcessor for SimpleFrameDiffer {
    fn process_frame(&mut self, frame: Frame) -> Result<Vec<MotionEvent>, ProcessingError> {
        let event = MotionEvent {
            timestamp: SystemTime::now(),
            source_id: frame.metadata.source_id,
            kind: MotionEventKind::FrameDifference,
            confidence: 0.0,
        };
        Ok(vec![event])
    }
}
