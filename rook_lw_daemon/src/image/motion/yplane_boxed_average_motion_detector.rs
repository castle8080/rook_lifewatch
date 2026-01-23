use rook_lw_models::image::MotionDetectionScore;

use super::YPlaneMotionDetector;
use crate::image::yplane::YPlane;
use crate::RookLWResult;

use std::collections::HashMap;

pub struct YPlaneBoxedAverageMotionDetector {
    pub box_size: usize,
    pub percentile: f32,
    pub percentile_threshold: f32,
}

impl YPlaneBoxedAverageMotionDetector {
    pub fn new(box_size: usize, percentile: f32, percentile_threshold: f32) -> Self {
        Self {
            box_size,
            percentile,
            percentile_threshold,
        }
    }
}

impl YPlaneMotionDetector for YPlaneBoxedAverageMotionDetector {
    fn detect_motion(&mut self, a: &YPlane<'_>, b: &YPlane<'_>) -> RookLWResult<MotionDetectionScore> {
        let mut scores = crate::image::motion::boxed_average::compute_boxed_averages(
            a,
            b,
            self.box_size,
        )?;

        scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let index = ((self.percentile.clamp(0.0, 1.0) * (scores.len() - 1) as f32).round()) as usize;
        let score = scores[index];

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
