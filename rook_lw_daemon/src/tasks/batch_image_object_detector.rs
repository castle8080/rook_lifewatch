use std::path::PathBuf;

use crate::image::frame::FrameResult;
use crate::image::object_detection::opencv_object_detector::OpenCVObjectDetector;

use tracing::{info, error};
use opencv::imgcodecs::imread;

pub struct BatchImageObjectDetector {
    image_dir: String,
    object_detector: OpenCVObjectDetector,
}

impl BatchImageObjectDetector {

    pub fn new(image_dir: String, object_detector: OpenCVObjectDetector) -> Self {
        BatchImageObjectDetector {
            image_dir,
            object_detector,
        }
    }

    pub fn run(&mut self) {
        let images_files = self.get_images_files();
        let unprocessed_files = self.get_unprocess_images_files(&images_files);

        info!(
            total_image_count = images_files.len(),
            unprocessed_image_count = unprocessed_files.len(),
            "Starting batch image object detection"
        );

        for (image_path, detection_file) in unprocessed_files {
            if let Err(e) = self.process_image(&image_path, &detection_file) {
                error!(image_path = %image_path.display(), error = %e, "Failed to process image");
            }
        }
    }

    fn get_images_files(&self) -> Vec<PathBuf> {
        let mut image_files = vec![];
        for entry in walkdir::WalkDir::new(&self.image_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("jpeg") {
                        image_files.push(path.to_owned());
                    }
                }
            }
        }
        image_files.sort();
        image_files
    }

    fn get_unprocess_images_files(&self, image_files: &Vec<PathBuf>) -> Vec<(PathBuf, PathBuf)> {
        let mut unprocessed_files = vec![];
        for image_path in image_files {
            // Get the detection file. Same name as image minus extension, plus .detections.json
            let detection_file = image_path.with_extension("detections.json");
            if !detection_file.exists() {
                unprocessed_files.push((image_path.to_owned(), detection_file));
            }
        }
        unprocessed_files
    }

    fn process_image(&mut self, image_path: &PathBuf, detection_file: &PathBuf) -> FrameResult<()> {
        // Load image using OpenCV
        let image_path_str = image_path.to_str()
            .ok_or_else(|| crate::image::frame::FrameError::ProcessingError("Invalid path".to_string()))?;
        
        info!(image_path = %image_path.display(), "Processing image for object detection");

        let img = imread(image_path_str, opencv::imgcodecs::IMREAD_COLOR)?;
        
        info!("Image loaded, running detection");

        let detections = self.object_detector.detect(&img)?;

        info!(detection_count = detections.len(), "Detections found");

        // Write detections to JSON file.
        let json_data = serde_json::to_string_pretty(&detections)?;
        std::fs::write(detection_file, json_data)?;

        info!(detection_file = %detection_file.display(), "Wrote detections file");

        Ok(())
    }

}

