use std::time::SystemTime;
use std::sync::mpsc::Receiver;

#[derive(Debug, Clone)]
pub struct MotionEvent {
    pub timestamp: SystemTime,
    pub kind: MotionEventKind,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub enum MotionEventKind {
    PirTrigger,
    FrameDifference,
}

pub trait MotionEventSource: Send {
    /// Return a receiver that will deliver `MotionEvent`s produced by this source.
    fn events(&self) -> Receiver<MotionEvent>;
}
