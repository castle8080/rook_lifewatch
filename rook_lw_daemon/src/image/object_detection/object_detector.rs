use crate::RookLWResult;

use rook_lw_models::image::DetectionResult;

use image::DynamicImage;

pub trait ObjectDetector: Send {

    /// Detect objects in the given image and return detection results.
    /// 
    /// The result includes detected objects and optional per-image embeddings
    /// for similarity search (if the model supports it).
    fn detect(
        &mut self,
        image: &DynamicImage,
    ) -> RookLWResult<DetectionResult>;

}