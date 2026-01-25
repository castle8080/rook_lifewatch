
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Clone)]
pub struct ImageInfoSearchOptions {
    pub start_date: Option<DateTime<FixedOffset>>,
    
    pub end_date: Option<DateTime<FixedOffset>>,

    #[serde(default)]
    pub detection_classes: Vec<String>,

    pub detection_class_confidence: Option<f32>,
    
    pub limit: Option<u32>,
    
    pub offset: Option<u32>,
}

