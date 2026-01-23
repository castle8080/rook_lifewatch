use rook_lw_models::image::MotionDetectionScore;

use super::YPlaneMotionDetector;
use crate::image::yplane::YPlane;
use crate::RookLWResult;

use std::collections::HashMap;

pub struct YPlaneMotionPercentileDetector {
    pub percentile: f32,
    pub percentile_threshold: f32,
}

impl YPlaneMotionPercentileDetector {
    pub fn new(percentile: f32, percentile_threshold: f32) -> Self {
        Self {
            percentile,
            percentile_threshold,
        }
    }
}

impl YPlaneMotionDetector for YPlaneMotionPercentileDetector {
    fn detect_motion(&mut self, a: &YPlane<'_>, b: &YPlane<'_>) -> RookLWResult<MotionDetectionScore> {
        let score = crate::image::motion::motion_percentile::get_motion_percentile(
            a,
            b,
            self.percentile,
            1,
        )?;

        let detected = score >= self.percentile_threshold;

        let mut properties = HashMap::new();
        properties.insert("percentile".to_string(), format!("{}", self.percentile));

        Ok(MotionDetectionScore {
            score,
            detected,
            properties,
        })
    }
}
