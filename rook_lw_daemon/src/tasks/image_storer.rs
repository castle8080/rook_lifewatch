use crate::RookLWResult;
use crate::image::conversions::dynamic_image_to_jpeg;
use crate::events::{CaptureEvent, StorageEvent, ImageProcessingEvent};
use crate::prodcon::{
    ProducerTask, ConsumerTask,
    OnProduceCallback, ProducerCallbacks
};

use rook_lw_models::image::ImageInfo;
use rook_lw_image_repo::image_info::ImageInfoRepository;

use std::path::PathBuf;

pub trait OnImageStoredCallback : OnProduceCallback<StorageEvent> {}

pub struct ImageStorer {
    storage_root: String,
    image_info_repository: Box<dyn ImageInfoRepository>,
    producer_callbacks: ProducerCallbacks<StorageEvent>,
}

impl ProducerTask<StorageEvent> for ImageStorer {
    fn get_producer_callbacks(&mut self) -> &mut ProducerCallbacks<StorageEvent> {
        &mut self.producer_callbacks
    }
}

impl ConsumerTask<ImageProcessingEvent> for ImageStorer {
    fn consume(&mut self, item: ImageProcessingEvent) -> RookLWResult<()> {
        self.process_capture_event(item)
    }
}

impl ImageStorer {
    pub fn new(storage_root: String, image_info_repository: Box<dyn ImageInfoRepository>) -> Self {
        Self { 
            storage_root, 
            image_info_repository,
            producer_callbacks: ProducerCallbacks::new(),
        }
    }

    fn process_capture_event(&mut self, image_processing_event: ImageProcessingEvent) -> RookLWResult<()> {

        let capture_event = &image_processing_event.capture_event;

        tracing::info!(
            event_id = %capture_event.event_id,
            capture_index = capture_event.capture_index,
            motion_score = %format!("{}", capture_event.motion_score),
            "Processing capture event"
        );

        let jpeg_data = dynamic_image_to_jpeg(&capture_event.image, Some(85))?;

        tracing::info!(
            event_id = %capture_event.event_id,
            capture_index = capture_event.capture_index,
            jpeg_len = jpeg_data.len(),
            "Encoded JPEG"
        );

        // The relative path from the image root.
        let image_path_rel = self.build_image_path(&capture_event);

        let image_path = PathBuf::from(&self.storage_root).join(&image_path_rel);
        if let Some(parent_dir) = image_path.parent() {
            std::fs::create_dir_all(parent_dir)?;
        }

        std::fs::write(&image_path, &jpeg_data)?;

        // Write detections to disk
        // Todo: remove once database store is working right.
        /*
        if let Some(detections) = &image_processing_event.detections {
            tracing::info!(
                event_id = %capture_event.event_id,
                capture_index = capture_event.capture_index,
                detection_count = detections.len(),
                image_path = %image_path.display(),
                "Stored image with detections"
            );

            let detections_file = image_path.with_extension("detections.json");
            let detections_json = serde_json::to_string_pretty(&detections)?;
            std::fs::write(&detections_file, &detections_json)?;
        }
        */

        // Save image info to repository
        let image_id = format!(
            "{}_{}",
            capture_event.event_id, capture_event.capture_index
        );

        let image_info = ImageInfo {
            image_id: image_id,
            event_id: capture_event.event_id.to_string(),
            event_timestamp: capture_event.event_timestamp,
            motion_score: capture_event.motion_score.clone(),
            capture_index: capture_event.capture_index,
            capture_timestamp: capture_event.capture_timestamp,
            detections: image_processing_event.detections.clone(),
            image_path: image_path_rel.to_string_lossy().to_string(),
        };

        self.image_info_repository.save_image_info(&image_info)?;

        let storage_event = StorageEvent {
            capture_event: capture_event.clone(),
            image_path,
        };

        self.produce(storage_event)?;

        Ok(())
    }

    fn build_image_path(&self, capture_event: &CaptureEvent) -> PathBuf {
        let date_dir = capture_event
            .capture_timestamp
            .format("%Y-%m-%d")
            .to_string();

        let timestamp = capture_event
            .capture_timestamp
            .format("%Y%m%d_%H%M%S%.3f")
            .to_string();

        // Keep a stable, fairly precise numeric format in filenames.
        let motion_score = format!("{:.9}", capture_event.motion_score.score);

        let filename = format!(
            "{timestamp}_{}_{}_{}.jpg",
            capture_event.event_id, capture_event.capture_index, motion_score
        );

        PathBuf::from(date_dir).join(filename)
    }
}