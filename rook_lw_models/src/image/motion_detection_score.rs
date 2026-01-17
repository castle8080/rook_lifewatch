
use std::{collections::HashMap, fmt};

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
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