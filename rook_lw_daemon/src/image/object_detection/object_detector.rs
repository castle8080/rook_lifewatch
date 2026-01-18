use crate::RookLWResult;

use rook_lw_models::image::Detection;

use image::DynamicImage;

pub trait ObjectDetector: Send {

    /// Detect objects in the given image and return a list of detections.
    fn detect(
        &mut self,
        image: &DynamicImage,
    ) -> RookLWResult<Vec<Detection>>;

}