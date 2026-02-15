use serde::{Deserialize, Serialize};

use super::Detection;

/// Result of object detection containing detections and optional per-image embeddings.
///
/// Embeddings represent the entire image (not individual detections) and are used
/// for similarity search across images.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DetectionResult {
    /// Individual object detections found in the image
    pub detections: Vec<Detection>,
    
    /// Optional embedding vector for the entire image.
    /// Only present if the model supports embeddings output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeddings: Option<Vec<f32>>,
}

impl DetectionResult {
    /// Create a new detection result with detections only (no embeddings)
    pub fn new(detections: Vec<Detection>) -> Self {
        Self {
            detections,
            embeddings: None,
        }
    }

    /// Create a new detection result with detections and embeddings
    pub fn with_embeddings(detections: Vec<Detection>, embeddings: Vec<f32>) -> Self {
        Self {
            detections,
            embeddings: Some(embeddings),
        }
    }

    /// Check if embeddings are available
    pub fn has_embeddings(&self) -> bool {
        self.embeddings.is_some()
    }
}
