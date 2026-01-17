
use serde::{Deserialize, Serialize};

/// A single object detection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub class_id: i32,
    pub class_name: String,
    pub confidence: f32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Detection {
    pub fn center(&self) -> (i32, i32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }
}