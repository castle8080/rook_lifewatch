use crate::RookLWResult;
use crate::image::conversions::dynamic_image_to_jpeg;
use crate::events::{CaptureEvent, StorageEvent, ImageProcessingEvent};
use crate::prodcon::{
    ProducerTask, ConsumerTask,
    OnProduceCallback, ProducerCallbacks
};

use rook_lw_models::image::ImageInfo;
use rook_lw_image_repo::image_info::ImageInfoRepository;
use rook_lw_image_repo::image_store::ImageStoreRepository;

pub trait OnImageStoredCallback : OnProduceCallback<StorageEvent> {}

pub struct ImageStorer {
    image_store_repository: Box<dyn ImageStoreRepository>,
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
    pub fn new(image_store_repository: Box<dyn ImageStoreRepository>, image_info_repository: Box<dyn ImageInfoRepository>) -> Self {
        Self { 
            image_store_repository,
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

        // Store image data
        self.image_store_repository.store(&image_path_rel, &jpeg_data)?;


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
            image_path: image_path_rel.to_string(),
        };

        self.image_info_repository.save_image_info(&image_info)?;

        let storage_event = StorageEvent {
            capture_event: capture_event.clone(),
            image_path: image_path_rel,
        };

        self.produce(storage_event)?;

        Ok(())
    }

    fn build_image_path(&self, capture_event: &CaptureEvent) -> String {
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
            "{date_dir}/{timestamp}_{}_{}_{}.jpg",
            capture_event.event_id, capture_event.capture_index, motion_score
        );

        filename
    }
}