use crate::{RookLWError, RookLWResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct AppConfiguration {
    // Frame Source Type
    pub frame_source: Option<String>,

    // Camera source
    pub camera_source: Option<String>,

    // Image directory
    pub image_directory: String,

    // Sqlite database path
    pub database_path: String,

    // Motion watcher settings
    pub use_motion_watcher: bool,
    pub motion_watcher_type: String,
    pub motion_watcher_count: u32,
    pub motion_watcher_round_interval_ms: u64,
    pub radar_gpio_pin: u32,
    pub radar_gpio_chip_path: Option<String>,

    // Image capturer settings
    pub image_capturer_capture_count: u32,
    pub image_capturer_capture_interval_ms: u64,

    // motion detector settings
    pub motion_detector_type: String,

    // Y Plane
    pub yplane_motion_percentile: f32,
    pub yplane_motion_percentile_threshold: f32,

    // Y Plane boxed average motion detector settings
    pub yplane_boxed_average_motion_detector_box_size: usize,
    pub yplane_boxed_average_motion_detector_percentile: f32,
    pub yplane_boxed_average_motion_detector_threshold: f32,

    // Y Plane rolling z settings
    pub use_yplane_rolling_z: bool,
    pub yplane_rolling_z_alpha: f64,
    pub yplane_rolling_z_threshold: f32,

    // object detector settings: opencv or onnx
    pub object_detector_type: String,

    // opencv object detector settings
    pub opencv_model_config_path: String,
    pub opencv_model_weights_path: String,
    pub opencv_model_names_path: String,
    pub opencv_model_confidence_threshold: f32,

    // onnx object detector settings
    pub onnx_model_path: String,
    pub onnx_model_names_path: String,
    pub onnx_model_confidence_threshold: f32,
}

impl Default for AppConfiguration {
    fn default() -> Self {
        AppConfiguration {
            frame_source: None,
            camera_source: None,
            image_directory: "var/images".into(),
            database_path: "var/db/image_info.db".into(),

            // motion watcher defaults
            use_motion_watcher: true,
            motion_watcher_type: "image_diff".into(),
            motion_watcher_count: 20,
            motion_watcher_round_interval_ms: 500,
            radar_gpio_pin: 27,
            radar_gpio_chip_path: None,

            // image capturer defaults
            image_capturer_capture_count: 5,
            image_capturer_capture_interval_ms: 100,

            // Which motion detector to use.
            motion_detector_type: "yplane_motion_percentile".into(),

            // y plane motion detector defaults
            yplane_motion_percentile: 0.95,
            yplane_motion_percentile_threshold: 0.02,

            // y plane boxed average motion detector defaults
            yplane_boxed_average_motion_detector_box_size: 100,
            yplane_boxed_average_motion_detector_percentile: 0.98,
            yplane_boxed_average_motion_detector_threshold: 0.02,

            // y plane rolling z defaults
            use_yplane_rolling_z: true,
            yplane_rolling_z_alpha: 0.05,
            yplane_rolling_z_threshold: 2.0,

            // object detector defaults
            object_detector_type: "opencv".into(),

            // opencv object detector defaults
            opencv_model_config_path: "models/yolov4-tiny.cfg".into(),
            opencv_model_weights_path: "models/yolov4-tiny.weights".into(),
            opencv_model_names_path: "models/coco.names".into(),
            opencv_model_confidence_threshold: 0.15,

            // onnx object detector defaults
            onnx_model_path: "models/yolov4-tiny.onnx".into(),
            onnx_model_names_path: "models/coco.names".into(),
            onnx_model_confidence_threshold: 0.15,
        }
    }
}

impl AppConfiguration {

    pub fn load(config_path: &str) -> RookLWResult<Self> {
        let config_str = std::fs::read_to_string(config_path)
            .map_err(|e| RookLWError::Config(format!("Failed to read app configuration file: {}", e)))?;
        let config: AppConfiguration = toml::from_str(&config_str)
            .map_err(|e| RookLWError::Config(format!("Failed to parse app configuration: {}", e)))?;
        Ok(config)
    }

}