use crate::image::yplane::YPlane;
use crate::RookLWResult;

use rook_lw_models::image::MotionDetectionScore;

pub trait YPlaneMotionDetector: Send {
    fn detect_motion(&mut self, a: &YPlane<'_>, b: &YPlane<'_>) -> RookLWResult<MotionDetectionScore>;
}
