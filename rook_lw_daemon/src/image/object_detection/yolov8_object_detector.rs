//! Object detection using YOLOv8 models with ONNX Runtime.
//!
//! YOLOv8 provides a more reliable and standardized interface compared to YOLOv4-tiny:
//! - Consistent preprocessing: RGB format, [0,1] normalization
//! - Standard output format: [batch, 84, 8400] where 84 = 4 bbox + 80 classes
//! - No objectness score needed - class scores are direct probabilities
//! - Better maintained and documented by Ultralytics

use crate::RookLWResult;
use crate::image::object_detection::ObjectDetector;
use rook_lw_models::image::{Detection, DetectionResult};

use anyhow::{Context, Result};
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};

use ort::{
    session::builder::GraphOptimizationLevel,
    session::Session,
};
use ndarray::Array4;
use image::GenericImageView;

/// Object detector using YOLOv8 models with ONNX Runtime.
///
/// YOLOv8 expects input shape [1, 3, 640, 640] and outputs [1, 84, 8400]
/// where 84 = 4 bbox coordinates (x_center, y_center, width, height) + 80 class scores.
///
/// Optionally supports dual-output models with embeddings for similarity search.
pub struct Yolov8ObjectDetector {
    session: Session,
    class_names: Vec<String>,
    confidence_threshold: f32,
    nms_threshold: f32,
    input_width: usize,
    input_height: usize,
}

impl Yolov8ObjectDetector {
    /// Create a new YOLOv8 object detector using ONNX Runtime.
    ///
    /// # Arguments
    ///
    /// * `model_path` - Path to the YOLOv8 .onnx model file (e.g., yolov8n.onnx)
    /// * `classes_path` - Path to class names file (one per line, e.g., coco.names)
    /// * `confidence_threshold` - Minimum confidence (0.0 to 1.0)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let detector = Yolov8ObjectDetector::new(
    ///     "yolov8n.onnx",
    ///     "coco.names",
    ///     0.25
    /// )?;
    /// ```
    pub fn new<P: AsRef<Path>>(
        model_path: P,
        classes_path: P,
        confidence_threshold: f32,
    ) -> RookLWResult<Self> {
        let model_path = model_path.as_ref();
        let classes_path = classes_path.as_ref();

        // Load ONNX model
        let session = Session::builder()
            .context("Failed to create session builder")?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .context("Failed to set optimization level")?
            .with_intra_threads(4)
            .context("Failed to set intra threads")?
            .commit_from_memory(std::fs::read(model_path).context("Failed to read model file")?.as_slice())
            .context("Failed to load ONNX model")?;

        // Load class names
        let file = File::open(classes_path).context("Failed to open classes file")?;
        let reader = BufReader::new(file);
        let class_names: Vec<String> = reader.lines()
            .collect::<std::io::Result<Vec<String>>>()
            .context("Failed to read class names")?;

        Ok(Self {
            session,
            class_names,
            confidence_threshold,
            nms_threshold: 0.45, // YOLOv8 default
            input_width: 640,    // YOLOv8 default input size
            input_height: 640,
        })
    }

    pub fn set_input_size(&mut self, width: usize, height: usize) -> &mut Self {
        self.input_width = width;
        self.input_height = height;
        self
    }

    pub fn set_nms_threshold(&mut self, threshold: f32) -> &mut Self {
        self.nms_threshold = threshold;
        self
    }

    pub fn class_names(&self) -> &[String] {
        &self.class_names
    }

    /// Detect objects in an image from a DynamicImage.
    ///
    /// # Arguments
    ///
    /// * `image` - Image from the `image` crate
    ///
    /// # Returns
    ///
    /// DetectionResult with detected objects and optional embeddings.
    pub fn detect(
        &mut self,
        image: &image::DynamicImage,
    ) -> RookLWResult<DetectionResult> {
        let (orig_width, orig_height) = image.dimensions();
        
        // Preprocess directly from DynamicImage
        let input_tensor = self.preprocess(image)?;
        
        // Run inference
        use ort::value::Tensor;
        let shape = input_tensor.shape().to_vec();
        let (data, _offset) = input_tensor.into_raw_vec_and_offset();
        let input_value = Tensor::from_array((shape.as_slice(), data))
            .context("Failed to create tensor")?;
        
        let outputs = self.session
            .run(ort::inputs![&input_value])
            .context("Failed to run ONNX inference")?;

        // Extract detections output
        let output_tuple = outputs[0].try_extract_tensor::<f32>()
            .context("Failed to extract output tensor")?;
        let (output_shape, output_data) = output_tuple;
        let shape_vec = output_shape.as_ref().to_vec();
        let data_vec = output_data.to_vec();
        
        // Extract embeddings if available
        let embeddings = if outputs.len() >= 2 {
            let emb_tuple = outputs[1].try_extract_tensor::<f32>()
                .context("Failed to extract embeddings tensor")?;
            let (_, emb_data) = emb_tuple;
            Some(emb_data.to_vec())
        } else {
            None
        };
        
        drop(outputs);

        self.post_process(&shape_vec, &data_vec, embeddings, orig_width as i32, orig_height as i32)
    }

    /// Preprocess image for YOLOv8 inference.
    ///
    /// YOLOv8 expects:
    /// - RGB format (not BGR)
    /// - Normalized to [0, 1] range
    /// - Shape: [1, 3, height, width] (CHW format)
    /// - Letterbox resizing (maintains aspect ratio with padding)
    fn preprocess(
        &self,
        image: &image::DynamicImage,
    ) -> Result<Array4<f32>> {
        let (width, height) = image.dimensions();
        let channels = 3;
        
        // Create input array with shape [1, 3, input_height, input_width]
        let mut input = Array4::<f32>::zeros((1, channels, self.input_height, self.input_width));

        // Simple resize (letterbox would be better but adds complexity)
        let x_ratio = width as f32 / self.input_width as f32;
        let y_ratio = height as f32 / self.input_height as f32;

        // Fill tensor in CHW format with [0, 1] normalization
        for out_y in 0..self.input_height {
            for out_x in 0..self.input_width {
                let src_x = ((out_x as f32 * x_ratio) as u32).min(width - 1);
                let src_y = ((out_y as f32 * y_ratio) as u32).min(height - 1);
                
                let pixel = image.get_pixel(src_x, src_y);
                let rgb = pixel.0;
                
                // YOLOv8: RGB format, normalized to [0, 1]
                input[[0, 0, out_y, out_x]] = rgb[0] as f32 / 255.0; // R
                input[[0, 1, out_y, out_x]] = rgb[1] as f32 / 255.0; // G
                input[[0, 2, out_y, out_x]] = rgb[2] as f32 / 255.0; // B
            }
        }

        Ok(input)
    }

    /// Post-process YOLOv8 output.
    ///
    /// YOLOv8 output shape: [batch, 84, 8400]
    /// - First 4 values per detection: x_center, y_center, width, height (normalized to input size)
    /// - Next 80 values: class scores (raw values, max is used for confidence)
    /// 
    /// Optional embeddings shape: [batch, channels] - feature vector for similarity search
    fn post_process(
        &self,
        output_shape: &[i64],
        output_data: &[f32],
        embeddings: Option<Vec<f32>>,
        image_width: i32,
        image_height: i32,
    ) -> RookLWResult<DetectionResult> {
        // YOLOv8 output: [1, 84, 8400]
        // 84 = 4 bbox coords + 80 class scores
        // 8400 = number of predictions
        
        if output_shape.len() != 3 {
            return Err(anyhow::anyhow!("Unexpected output shape: {:?}", output_shape).into());
        }

        let num_classes = (output_shape[1] - 4) as usize; // Should be 80 for COCO
        let num_predictions = output_shape[2] as usize;   // Should be 8400

        let mut class_ids = Vec::new();
        let mut confidences = Vec::new();
        let mut boxes = Vec::new();

        // Calculate scale factors
        let x_scale = image_width as f32 / self.input_width as f32;
        let y_scale = image_height as f32 / self.input_height as f32;

        // Process each prediction
        for i in 0..num_predictions {
            // YOLOv8 output is transposed: data is in [84, 8400] layout
            // Access pattern: output[row * num_predictions + col]
            
            // Get bbox coordinates (first 4 rows)
            let x_center = output_data[0 * num_predictions + i] * x_scale;
            let y_center = output_data[1 * num_predictions + i] * y_scale;
            let width = output_data[2 * num_predictions + i] * x_scale;
            let height = output_data[3 * num_predictions + i] * y_scale;

            // Find best class (rows 4 through 83)
            let mut max_score = 0.0_f32;
            let mut best_class_id = 0;
            
            for class_id in 0..num_classes {
                let score = output_data[(4 + class_id) * num_predictions + i];
                if score > max_score {
                    max_score = score;
                    best_class_id = class_id;
                }
            }

            // YOLOv8: confidence is just the max class score (no objectness)
            if max_score > self.confidence_threshold {
                // Convert from center format to corner format
                let x = x_center - width / 2.0;
                let y = y_center - height / 2.0;

                boxes.push((x, y, width, height));
                confidences.push(max_score);
                class_ids.push(best_class_id as i32);
            }
        }

        // Apply NMS (Non-Maximum Suppression)
        let indices = self.apply_nms(&boxes, &confidences, &class_ids);

        let mut detections = Vec::new();
        for &idx in &indices {
            let (x, y, width, height) = boxes[idx];
            let class_id = class_ids[idx];
            let confidence = confidences[idx];

            let class_name = self.class_names
                .get(class_id as usize)
                .cloned()
                .unwrap_or_else(|| format!("Unknown({})", class_id));

            detections.push(Detection {
                class_id,
                class_name,
                confidence,
                x: x.max(0.0) as i32,
                y: y.max(0.0) as i32,
                width: width.max(0.0) as i32,
                height: height.max(0.0) as i32,
            });
        }

        Ok(DetectionResult {
            detections,
            embeddings,
        })
    }

    /// Apply class-aware NMS to filter overlapping detections.
    ///
    /// YOLOv8 typically uses class-aware NMS, meaning boxes from different
    /// classes don't suppress each other.
    fn apply_nms(&self, boxes: &[(f32, f32, f32, f32)], scores: &[f32], class_ids: &[i32]) -> Vec<usize> {
        if boxes.is_empty() {
            return Vec::new();
        }

        // Sort by score descending
        let mut indices: Vec<usize> = (0..boxes.len()).collect();
        indices.sort_by(|&a, &b| {
            scores[b].partial_cmp(&scores[a]).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut keep = Vec::new();
        let mut suppressed = vec![false; boxes.len()];

        for &idx in &indices {
            if suppressed[idx] {
                continue;
            }

            keep.push(idx);

            let box_a = boxes[idx];
            let class_a = class_ids[idx];
            
            for &other_idx in &indices {
                if suppressed[other_idx] || other_idx == idx {
                    continue;
                }

                let class_b = class_ids[other_idx];
                
                // Only suppress boxes of the same class
                if class_a != class_b {
                    continue;
                }

                let box_b = boxes[other_idx];
                let iou = self.compute_iou(box_a, box_b);
                
                if iou > self.nms_threshold {
                    suppressed[other_idx] = true;
                }
            }
        }

        keep
    }

    /// Compute Intersection over Union (IoU) between two bounding boxes.
    ///
    /// IoU measures the overlap between two boxes:
    /// - 0.0 = no overlap
    /// - 1.0 = perfect overlap
    /// 
    /// Used in NMS to determine if two detections are for the same object.
    fn compute_iou(&self, box_a: (f32, f32, f32, f32), box_b: (f32, f32, f32, f32)) -> f32 {
        let (x1, y1, w1, h1) = box_a;
        let (x2, y2, w2, h2) = box_b;

        // Convert from (x, y, w, h) to (x_min, y_min, x_max, y_max)
        let x1_max = x1 + w1;
        let y1_max = y1 + h1;
        let x2_max = x2 + w2;
        let y2_max = y2 + h2;

        // Find intersection rectangle bounds
        let inter_x1 = x1.max(x2);
        let inter_y1 = y1.max(y2);
        let inter_x2 = x1_max.min(x2_max);
        let inter_y2 = y1_max.min(y2_max);

        // Calculate intersection area (0 if boxes don't overlap)
        let inter_width = (inter_x2 - inter_x1).max(0.0);
        let inter_height = (inter_y2 - inter_y1).max(0.0);
        let inter_area = inter_width * inter_height;

        // Calculate union area: area(A) + area(B) - intersection
        let box_a_area = w1 * h1;
        let box_b_area = w2 * h2;
        let union_area = box_a_area + box_b_area - inter_area;

        // IoU = intersection / union
        if union_area > 0.0 {
            inter_area / union_area
        } else {
            0.0
        }
    }
}

impl ObjectDetector for Yolov8ObjectDetector {
    fn detect(
        &mut self,
        image: &image::DynamicImage,
    ) -> RookLWResult<DetectionResult> {
        self.detect(image)
    }
}
