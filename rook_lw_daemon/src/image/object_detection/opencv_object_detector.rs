/// Object detection using YOLO (Darknet) models with OpenCV DNN module.

use crate::{RookLWResult, RookLWError};
use super::ObjectDetector;

use rook_lw_models::image::{Detection, DetectionResult};

use opencv::{
    core::{Mat, Scalar, Size, Rect, Vector},
    dnn::{Net, read_net_from_darknet, blob_from_image, DNN_BACKEND_DEFAULT, DNN_TARGET_CPU, nms_boxes},
    prelude::{NetTrait, NetTraitConst, MatTraitConst},
};
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use anyhow::Context;

// OpenCV Net is thread-safe for inference, so this is safe for our use case
unsafe impl Send for OpenCVObjectDetector {}

/// Object detector using YOLO models.
pub struct OpenCVObjectDetector {
    net: Net,
    class_names: Vec<String>,
    confidence_threshold: f32,
    nms_threshold: f32,
    input_size: i32,
}

impl OpenCVObjectDetector {
    /// Create a new YOLO object detector.
    ///
    /// # Arguments
    ///
    /// * `cfg_path` - Path to the YOLO .cfg configuration file
    /// * `weights_path` - Path to the YOLO .weights file
    /// * `classes_path` - Path to class names file (one per line)
    /// * `confidence_threshold` - Minimum confidence (0.0 to 1.0)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let detector = OpenCVObjectDetector::new(
    ///     "yolov4-tiny.cfg",
    ///     "yolov4-tiny.weights",
    ///     "coco.names",
    ///     0.5
    /// )?;
    /// ```
    pub fn new<P: AsRef<Path>>(
        cfg_path: P,
        weights_path: P,
        classes_path: P,
        confidence_threshold: f32,
    ) -> RookLWResult<Self> {
        let cfg_path = cfg_path.as_ref();
        let weights_path = weights_path.as_ref();
        let classes_path = classes_path.as_ref();

        // Load YOLO model
        let mut net = read_net_from_darknet(
            cfg_path.to_str().context("Invalid config path")?,
            weights_path.to_str().context("Invalid weights path")?
        ).context("Failed to load YOLO model")?;

        net.set_preferable_backend(DNN_BACKEND_DEFAULT)
            .context("Failed to set backend")?;
        net.set_preferable_target(DNN_TARGET_CPU)
            .context("Failed to set target")?;

        // Load class names
        let file = File::open(classes_path).context("Failed to open classes file")?;
        let reader = BufReader::new(file);
        let class_names: Vec<String> = reader.lines()
            .collect::<std::io::Result<Vec<String>>>()
            .context("Failed to read class names")?;

        Ok(Self {
            net,
            class_names,
            confidence_threshold,
            nms_threshold: 0.4,
            input_size: 416,
        })
    }

    pub fn set_input_size(&mut self, size: i32) -> &mut Self {
        self.input_size = size;
        self
    }

    pub fn set_nms_threshold(&mut self, threshold: f32) -> &mut Self {
        self.nms_threshold = threshold;
        self
    }

    pub fn class_names(&self) -> &[String] {
        &self.class_names
    }

    /// Detect objects in an image.
    pub fn detect(&mut self, image: &Mat) -> RookLWResult<DetectionResult> {
        let image_width = image.cols();
        let image_height = image.rows();

        // YOLO preprocessing: scale to [0,1], no mean, RGB
        let blob = blob_from_image(
            image,
            1.0 / 255.0,
            Size::new(self.input_size, self.input_size),
            Scalar::default(),
            true,  // Swap RB (BGR to RGB)
            false,
            opencv::core::CV_32F,
        ).context("Failed to create blob")?;

        self.net.set_input(&blob, "", 1.0, Scalar::default())
            .context("Failed to set input")?;

        // Forward pass
        let output_layer_names = self.net.get_unconnected_out_layers_names()
            .context("Failed to get output layers")?;
        
        let mut outputs = Vector::<Mat>::new();
        self.net.forward(&mut outputs, &output_layer_names)
            .context("Failed to forward")?;

        // Post-process
        self.post_process(&outputs, image_width, image_height)
    }

    fn post_process(&self, outputs: &Vector<Mat>, image_width: i32, image_height: i32) -> RookLWResult<DetectionResult> {
        let mut class_ids = Vector::<i32>::new();
        let mut confidences = Vector::<f32>::new();
        let mut boxes = Vector::<Rect>::new();

        // Process each output layer
        for i in 0..outputs.len() {
            let output = outputs.get(i).context("Failed to get output")?;
            let rows = output.rows();
            let cols = output.cols();

            for j in 0..rows {
                let scores_start = 5;
                let mut max_score = 0.0f32;
                let mut class_id = 0i32;
                
                // Find best class
                for k in scores_start..cols {
                    let score = *output.at_2d::<f32>(j, k).context("Failed to read score")?;
                    if score > max_score {
                        max_score = score;
                        class_id = (k - scores_start) as i32;
                    }
                }

                let confidence = max_score;
                if confidence > self.confidence_threshold {
                    // Get box (center_x, center_y, width, height - normalized [0,1])
                    let center_x = *output.at_2d::<f32>(j, 0)? * image_width as f32;
                    let center_y = *output.at_2d::<f32>(j, 1)? * image_height as f32;
                    let width = *output.at_2d::<f32>(j, 2)? * image_width as f32;
                    let height = *output.at_2d::<f32>(j, 3)? * image_height as f32;

                    let x = (center_x - width / 2.0) as i32;
                    let y = (center_y - height / 2.0) as i32;

                    boxes.push(Rect::new(x, y, width as i32, height as i32));
                    confidences.push(confidence);
                    class_ids.push(class_id);
                }
            }
        }

        // NMS
        let mut indices = Vector::<i32>::new();
        nms_boxes(&boxes, &confidences, self.confidence_threshold, self.nms_threshold, &mut indices, 1.0, 0)
            .context("NMS failed")?;

        let mut detections = Vec::new();
        for i in 0..indices.len() {
            let idx = indices.get(i)? as usize;
            let bbox = boxes.get(idx)?;
            let class_id = class_ids.get(idx)?;
            let confidence = confidences.get(idx)?;

            let class_name = self.class_names
                .get(class_id as usize)
                .cloned()
                .unwrap_or_else(|| format!("Unknown({})", class_id));

            detections.push(Detection {
                class_id,
                class_name,
                confidence,
                x: bbox.x.max(0),
                y: bbox.y.max(0),
                width: bbox.width.max(0),
                height: bbox.height.max(0),
            });
        }

        Ok(DetectionResult::new(detections))
    }

}

impl ObjectDetector for OpenCVObjectDetector {
    /// Detect objects in the given image and return a list of detections.
    fn detect(
        &mut self,
        image: &image::DynamicImage,
    ) -> RookLWResult<DetectionResult> {
        let mat_ref = dynamic_image_to_mat(image)
            .map_err(|e| RookLWError::Image(format!("OpenCV conversion error: {e}")))?;
        self.detect(&mat_ref)
    }
}

/// Convert a DynamicImage to an OpenCV Mat (CV_8UC3, RGB order)
fn dynamic_image_to_mat(img: &image::DynamicImage) -> opencv::Result<Mat> {
    let rgb = img.to_rgb8();
    let (_width, height) = rgb.dimensions();
    let boxed = Mat::from_slice(rgb.as_raw())?;
    let reshaped = boxed.reshape(3, height as i32)?;
    reshaped.try_clone()
}