use crate::RookLWResult;
use crate::events::ImageProcessingEvent;
use crate::tasks::motion_watcher::MotionWatcher;
use crate::tasks::image_storer::ImageStorer;
use crate::tasks::image_detector::ImageDetector;
use crate::prodcon::{ProducerTask, ConsumerTask};

use tracing::error;

pub struct App {
    motion_watcher: Box<dyn MotionWatcher>,
    image_storer: ImageStorer,
    image_detector: ImageDetector,
}

impl App {

    pub fn new(
        motion_watcher: Box<dyn MotionWatcher>,
        image_storer: ImageStorer,
        image_detector: ImageDetector) -> Self {
        
        Self {
            motion_watcher,
            image_storer,
            image_detector,
        }
    }

    pub fn run(self) -> RookLWResult<()> {
        // ImageDiffMotionWatcher produces CaptureEvents; a separate worker receives and processes them.
        // Bounded provides backpressure so we don't buffer unbounded image data.
        let (motion_detected_tx, motion_detected_rx) = crossbeam_channel::bounded::<ImageProcessingEvent>(64);

        // ImageDetector produces ImageProcessingEvents; ImageStorer receives and processes them.
        let (object_detected_tx, object_detected_rx) = crossbeam_channel::bounded::<ImageProcessingEvent>(64);

        let App { mut motion_watcher, image_storer, mut image_detector } = self;

        motion_watcher.connect(motion_detected_tx);
        image_detector.connect(object_detected_tx);

        let handles = vec![
            motion_watcher.start(),
            image_detector.start_listener(motion_detected_rx),
            image_storer.start_listener(object_detected_rx),
        ];

        for handle in handles {
            match handle.join() {
                Ok(result) => {
                    if let Err(e) = result {
                        error!(error = %e, "Task failed");
                    }
                },
                Err(e) => {
                    error!(error = ?e, "Task panicked");
                }
            }
        }

        Ok(())
    }
}