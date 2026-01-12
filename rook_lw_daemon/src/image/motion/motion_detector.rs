
use std::collections::HashMap;
use std::fmt;

use crate::image::yplane::YPlane;
use crate::image::frame::FrameResult;
use crate::stats::rollingz::RollingZ;

#[derive(Clone, Debug)]
pub struct MotionDetectionScore {
    pub score: f32,
    pub detected: bool,
    pub properties: HashMap<String, String>,
}

impl fmt::Display for MotionDetectionScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MotionDetectionScore {{ score: {:.4}, detected: {}, properties: {:?} }}",
            self.score, self.detected, self.properties
        )
    }
}

pub trait YPlaneMotionDetector: Send {
    fn detect_motion(&mut self, a: &YPlane<'_>, b: &YPlane<'_>) -> FrameResult<MotionDetectionScore>;
}

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
    fn detect_motion(&mut self, a: &YPlane<'_>, b: &YPlane<'_>) -> FrameResult<MotionDetectionScore> {
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

pub struct YPlaneRollingZMotionDetector<T: YPlaneMotionDetector> {
    rolling_z: RollingZ,
    z_threshold: f32,
    detector: T,
}

impl<T: YPlaneMotionDetector> YPlaneRollingZMotionDetector<T> {
    pub fn new(detector: T, rolling_z_alpha: f64, z_threshold: f32) -> FrameResult<Self> {
        Ok(Self {
            rolling_z: RollingZ::new(rolling_z_alpha),
            z_threshold,
            detector,
        })
    }
}

impl<T: YPlaneMotionDetector> YPlaneMotionDetector for YPlaneRollingZMotionDetector<T> {
    fn detect_motion(&mut self, a: &YPlane<'_>, b: &YPlane<'_>) -> FrameResult<MotionDetectionScore> {
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
