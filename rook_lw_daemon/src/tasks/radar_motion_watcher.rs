use crate::RookLWResult;
use crate::image::conversions::frame_to_dynamic_image;
use crate::image::frame::FrameSource;
use crate::events::{CaptureEvent, ImageProcessingEvent};
use crate::prodcon::{ProducerTask, ProducerCallbacks};
use crate::error::RookLWError;

use rook_lw_models::image::MotionDetectionScore;

use std::sync::Arc;
use std::time::Duration;
use std::thread::{JoinHandle, sleep, spawn};
use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use tracing::info;

use uuid::Uuid;

use gpiod::{Chip, Options, EdgeDetect, Input};

struct RadarDetectionResult {
    pub event_id: Uuid,
    pub event_timestamp: DateTime<FixedOffset>,
    pub capture_events: Vec<CaptureEvent>,
}

pub struct RadarMotionWatcher {
    frame_source: Box<dyn FrameSource + Send>,
    producer_callbacks: ProducerCallbacks<ImageProcessingEvent>,
    gpio_chip_path: Option<String>,
    gpio_pin: u32,
    capture_count: u32,
    capture_interval: Duration,
    round_interval: Duration,
}

impl ProducerTask<ImageProcessingEvent> for RadarMotionWatcher {
    fn get_producer_callbacks(&mut self) -> &mut ProducerCallbacks<ImageProcessingEvent> {
        &mut self.producer_callbacks
    }
}

impl RadarMotionWatcher {

    pub fn new(
        frame_source: Box<dyn FrameSource + Send>,
        gpio_chip_path: Option<String>,
        gpio_pin: u32,
        capture_count: u32,
        capture_interval: Duration,
        round_interval: Duration,
    ) -> Self {
        Self {
            frame_source,
            producer_callbacks: ProducerCallbacks::new(),
            gpio_chip_path,
            gpio_pin,
            capture_count,
            capture_interval,
            round_interval,
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
        self.frame_source.start()?;

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
            sleep(self.round_interval);
        }

        // Note: GPIO lines and chip will be closed when dropped
        // self.frame_source.stop()?;
        // Ok(())
    }

    fn on_image_processing_event(&mut self, event: ImageProcessingEvent) -> RookLWResult<()> {
        self.produce(event)
    }

    fn on_radar_detected(&mut self, result: RadarDetectionResult) -> RookLWResult<()> {
        // Capture images after radar trigger
        for capture_index in 0..self.capture_count {

            let capture_event: CaptureEvent = {
                let frame = self.frame_source.next_frame()?;
                CaptureEvent {
                    event_id: result.event_id,
                    event_timestamp: result.event_timestamp,
                    motion_score: MotionDetectionScore {
                        detected: true,
                        score: 1.0,
                        properties: HashMap::new(),
                    },
                    capture_index,
                    capture_timestamp: chrono::Local::now().into(),
                    image: Arc::new(frame_to_dynamic_image(&*frame)?),
                }
            };

            self.on_image_processing_event(ImageProcessingEvent {
                capture_event: capture_event.clone(),
                detections: None,
            })?;

            sleep(self.capture_interval);
        }

        Ok(())
    }

    fn wait_for_radar_trigger(&mut self, lines: &mut gpiod::Lines<Input>) -> RookLWResult<Option<RadarDetectionResult>> {
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
                let result = RadarDetectionResult {
                    event_id,
                    event_timestamp,
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
