
pub mod opencv_object_detector;
pub mod onnx_object_detector;

use crate::image::frame::FrameResult;
use image::DynamicImage;

use rook_lw_models::image::Detection;

pub trait ObjectDetector: Send {

    /// Detect objects in the given image and return a list of detections.
    fn detect(
        &mut self,
        image: &DynamicImage,
    ) -> FrameResult<Vec<Detection>>;

}