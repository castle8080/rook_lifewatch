use crate::RookLWResult;
use crate::events::ImageProcessingEvent;

use std::thread::JoinHandle;
use crossbeam_channel::Sender;

pub trait MotionWatcher: Send {
    fn connect(&mut self, sender: Sender<ImageProcessingEvent>);
    fn start(self: Box<Self>) -> JoinHandle<RookLWResult<()>>;
}
