use crate::RookLWResult;
use crate::events::{ImageProcessingEvent, MotionDetectionEvent};
use crate::prodcon::{ProducerTask, ProducerCallbacks};
use crate::tasks::image_capturer::ImageCapturer;
use crate::tasks::motion_watcher::MotionWatcher;
use crate::error::RookLWError;

use rook_lw_models::image::MotionDetectionScore;

use std::thread::{JoinHandle, spawn};
use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use tracing::info;

use uuid::Uuid;

use gpiod::{Chip, Options, EdgeDetect, Input};

impl MotionWatcher for RadarMotionWatcher {
    fn connect(&mut self, sender: crossbeam_channel::Sender<ImageProcessingEvent>) {
        ProducerTask::connect(self, sender);
    }

    fn start(self: Box<Self>) -> JoinHandle<RookLWResult<()>> {
        RadarMotionWatcher::start(*self)
    }
}

pub struct RadarMotionWatcher {
    gpio_chip_path: Option<String>,
    gpio_pin: u32,
    image_capturer: ImageCapturer,
}

impl ProducerTask<ImageProcessingEvent> for RadarMotionWatcher {
    fn get_producer_callbacks(&mut self) -> &mut ProducerCallbacks<ImageProcessingEvent> {
        self.image_capturer.get_producer_callbacks()
    }
}

impl RadarMotionWatcher {

    pub fn new(
        gpio_chip_path: Option<String>,
        gpio_pin: u32,
        image_capturer: ImageCapturer,
    ) -> Self {
        Self {
            gpio_chip_path,
            gpio_pin,
            image_capturer,
        }
    }

    pub fn start(mut self) -> JoinHandle<RookLWResult<()>> {
        spawn(move || {
            match self.run() {
                Ok(_) => {
                    info!("Radar motion watcher exiting normally");
                    Ok(())
                },
                Err(e) => {
                    info!(error = %e, "Radar motion watcher exiting with error");
                    Err(e)
                }
            }
        })
    }

    pub fn run(&mut self) -> RookLWResult<()> {
        info!("Starting radar motion watcher");

        // Initialize GPIO chip and line for radar input
        let chip_path = self.gpio_chip_path.as_deref().unwrap_or("/dev/gpiochip0");
        let chip = Chip::new(chip_path)
            .map_err(|e| RookLWError::Initialization(format!("Failed to open GPIO chip {}: {}", chip_path, e)))?;

        let options = Options::input([self.gpio_pin])
            .edge(EdgeDetect::Both)
            .consumer("radar-motion-watcher");

        let mut lines = chip.request_lines(options)
            .map_err(|e| RookLWError::Initialization(format!("Failed to request GPIO lines: {}", e)))?;

        info!(
            chip_path = %chip_path,
            gpio_pin = self.gpio_pin,
            "GPIO radar input configured"
        );

        loop {
            match self.wait_for_radar_trigger(&mut lines)? {
                Some(radar_detection_result) => {
                    self.on_radar_detected(radar_detection_result)?;
                },
                None => {
                    // Timeout or no event, continue waiting
                }
            }
        }

        // Note: GPIO lines and chip will be closed when dropped
        // Ok(())
    }

    fn on_radar_detected(&mut self, result: MotionDetectionEvent) -> RookLWResult<()> {
        self.image_capturer.on_motion_detected(result)
    }

    fn wait_for_radar_trigger(&mut self, lines: &mut gpiod::Lines<Input>) -> RookLWResult<Option<MotionDetectionEvent>> {
        // Try to read an event from the GPIO line - this will block until an event occurs
        match lines.read_event() {
            Ok(event) => {
                let event_id = Uuid::new_v4();
                let event_timestamp: DateTime<FixedOffset> = chrono::Local::now().into();

                info!(
                    event_id = %event_id,
                    line = event.line,
                    edge = ?event.edge,
                    "Radar trigger detected"
                );

                // Return trigger event without capturing images
                // Image capture happens later in on_radar_detected()
                let result = MotionDetectionEvent {
                    event_id,
                    event_timestamp,
                    motion_score: MotionDetectionScore {
                        detected: true,
                        score: 1.0,
                        properties: HashMap::new(),
                    },
                    capture_events: Vec::new(),
                };

                return Ok(Some(result));
            },
            Err(e) => {
                Err(RookLWError::Initialization(format!("Failed to read GPIO event: {}", e)))
            }
        }
    }
}
