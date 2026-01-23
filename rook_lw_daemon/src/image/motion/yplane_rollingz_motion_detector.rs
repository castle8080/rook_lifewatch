use rook_lw_models::image::MotionDetectionScore;

use super::YPlaneMotionDetector;
use crate::image::yplane::YPlane;
use crate::RookLWResult;
use crate::stats::RollingZ;

pub struct YPlaneRollingZMotionDetector<T: YPlaneMotionDetector> {
    rolling_z: RollingZ,
    z_threshold: f32,
    detector: T,
}

impl<T: YPlaneMotionDetector> YPlaneRollingZMotionDetector<T> {
    pub fn new(detector: T, rolling_z_alpha: f64, z_threshold: f32) -> RookLWResult<Self> {
        Ok(Self {
            rolling_z: RollingZ::new(rolling_z_alpha),
            z_threshold,
            detector,
        })
    }
}

impl<T: YPlaneMotionDetector> YPlaneMotionDetector for YPlaneRollingZMotionDetector<T> {
    fn detect_motion(&mut self, a: &YPlane<'_>, b: &YPlane<'_>) -> RookLWResult<MotionDetectionScore> {
        let mut result = self.detector.detect_motion(a, b)?;
        let z_score = self.rolling_z.update(result.score as f64) as f32;
        let detected = result.detected && z_score >= self.z_threshold;

        result.properties.insert("rolling_z".to_string(), format!("{}", z_score));
        result.properties.insert("rolling_z_underlying_score".to_string(), format!("{}", result.score));
        result.properties.insert("rolling_z_underlying_detected".to_string(), format!("{}", result.detected));

        Ok(MotionDetectionScore {
            score: z_score,
            detected,
            properties: result.properties,
        })
    }
}