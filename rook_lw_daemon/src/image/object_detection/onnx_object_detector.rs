//! Object detection using YOLO models with ONNX Runtime.

use crate::image::frame::FrameResult;
use crate::image::object_detection::ObjectDetector;
use super::Detection;

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

/// Object detector using YOLO models with ONNX Runtime.
pub struct OnnxObjectDetector {
    session: Session,
    class_names: Vec<String>,
    confidence_threshold: f32,
    nms_threshold: f32,
    input_width: usize,
    input_height: usize,
}

impl OnnxObjectDetector {
    /// Create a new YOLO object detector using ONNX Runtime.
    ///
    /// # Arguments
    ///
    /// * `model_path` - Path to the YOLO .onnx model file
    /// * `classes_path` - Path to class names file (one per line)
    /// * `confidence_threshold` - Minimum confidence (0.0 to 1.0)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let detector = OnnxObjectDetector::new(
    ///     "yolov4-tiny.onnx",
    ///     "coco.names",
    ///     0.5
    /// )?;
    /// ```
    pub fn new<P: AsRef<Path>>(
        model_path: P,
        classes_path: P,
        confidence_threshold: f32,
    ) -> FrameResult<Self> {
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
            nms_threshold: 0.4,
            input_width: 416,
            input_height: 416,
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
    /// This method efficiently preprocesses the image without requiring an intermediate
    /// RGB buffer allocation.
    ///
    /// # Arguments
    ///
    /// * `image` - Image from the `image` crate (supports JPEG, PNG, etc.)
    ///
    /// # Returns
    ///
    /// Vector of detected objects with bounding boxes and confidence scores.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let img = image::open("photo.jpg")?;
    /// let detections = detector.detect(&img)?;
    /// ```
    pub fn detect(
        &mut self,
        image: &image::DynamicImage,
    ) -> FrameResult<Vec<Detection>> {
        let (width, height) = image.dimensions();
        
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

        let output_tuple = outputs[0].try_extract_tensor::<f32>()
            .context("Failed to extract output tensor")?;
        let (output_shape, output_data) = output_tuple;
        let shape_vec = output_shape.as_ref().to_vec();
        let data_vec = output_data.to_vec();
        drop(outputs);

        self.post_process(&shape_vec, &data_vec, width as i32, height as i32)
    }

    /// Preprocess directly from a DynamicImage without allocating an RGB buffer.
    ///
    /// This efficiently accesses pixels on-demand via get_pixel() instead of
    /// materializing the entire image in RGB format first.
    fn preprocess(
        &self,
        image: &image::DynamicImage,
    ) -> Result<Array4<f32>> {
        let (width, height) = image.dimensions();
        let channels = 3;
        
        // Create input array with shape [1, 3, input_height, input_width]
        let mut input = Array4::<f32>::zeros((1, channels, self.input_height, self.input_width));

        // Resize ratios
        let x_ratio = width as f32 / self.input_width as f32;
        let y_ratio = height as f32 / self.input_height as f32;

        // Iterate over output tensor dimensions
        for out_y in 0..self.input_height {
            for out_x in 0..self.input_width {
                // Calculate source coordinates
                let src_x = ((out_x as f32 * x_ratio) as u32).min(width - 1);
                let src_y = ((out_y as f32 * y_ratio) as u32).min(height - 1);
                
                // Get pixel directly from DynamicImage (no RGB buffer allocation!)
                let pixel = image.get_pixel(src_x, src_y);
                let rgb = pixel.0; // [r, g, b, a] - we ignore alpha
                
                // Normalize to [0, 1] and store in CHW format
                input[[0, 0, out_y, out_x]] = rgb[0] as f32 / 255.0; // R
                input[[0, 1, out_y, out_x]] = rgb[1] as f32 / 255.0; // G
                input[[0, 2, out_y, out_x]] = rgb[2] as f32 / 255.0; // B
            }
        }

        Ok(input)
    }

    fn post_process(
        &self,
        output_shape: &[i64],
        output_data: &[f32],
        image_width: i32,
        image_height: i32,
    ) -> FrameResult<Vec<Detection>> {
        // YOLOv4 output format: [batch, num_detections, 5+num_classes]
        // Each detection: [center_x, center_y, width, height, objectness, class_scores...]
        let shape = output_shape;
        
        if shape.len() < 3 {
            return Err(anyhow::anyhow!("Unexpected output shape: {:?}", shape).into());
        }

        let num_detections = shape[1] as usize;
        let num_attrs = shape[2] as usize;
        
        if num_attrs < 5 {
            return Err(anyhow::anyhow!("Invalid output attributes: {}", num_attrs).into());
        }

        let mut class_ids = Vec::new();
        let mut confidences = Vec::new();
        let mut boxes = Vec::new();

        // Process each detection
        for i in 0..num_detections {
            let detection_start = i * num_attrs;
            let detection = &output_data[detection_start..detection_start + num_attrs];

            // Find best class (apply softmax to class scores)
            let scores = &detection[5..];
            let mut exp_scores = Vec::with_capacity(scores.len());
            let max_logit = scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let mut sum_exp = 0.0;
            for &s in scores {
                let exp_s = (s - max_logit).exp();
                exp_scores.push(exp_s);
                sum_exp += exp_s;
            }
            let softmax_scores: Vec<f32> = exp_scores.iter().map(|&e| e / sum_exp).collect();
            let (class_id, &max_score) = softmax_scores
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((0, &0.0));

            let objectness = detection[4];
            let confidence = objectness * max_score;

            if confidence > self.confidence_threshold {
                // Get box (center_x, center_y, width, height - normalized [0,1])
                let center_x = detection[0] * image_width as f32;
                let center_y = detection[1] * image_height as f32;
                let width = detection[2] * image_width as f32;
                let height = detection[3] * image_height as f32;

                let x = center_x - width / 2.0;
                let y = center_y - height / 2.0;

                boxes.push((x, y, width, height));
                confidences.push(confidence);
                class_ids.push(class_id as i32);
            }
        }

        // Apply NMS (Non-Maximum Suppression)
        let indices = self.apply_nms(&boxes, &confidences);

        let mut detections = Vec::new();
        for idx in indices {
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

        Ok(detections)
    }

    fn apply_nms(&self, boxes: &[(f32, f32, f32, f32)], scores: &[f32]) -> Vec<usize> {
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
            for &other_idx in &indices {
                if suppressed[other_idx] || other_idx == idx {
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

    fn compute_iou(&self, box_a: (f32, f32, f32, f32), box_b: (f32, f32, f32, f32)) -> f32 {
        let (x1, y1, w1, h1) = box_a;
        let (x2, y2, w2, h2) = box_b;

        let x1_max = x1 + w1;
        let y1_max = y1 + h1;
        let x2_max = x2 + w2;
        let y2_max = y2 + h2;

        let inter_x1 = x1.max(x2);
        let inter_y1 = y1.max(y2);
        let inter_x2 = x1_max.min(x2_max);
        let inter_y2 = y1_max.min(y2_max);

        let inter_width = (inter_x2 - inter_x1).max(0.0);
        let inter_height = (inter_y2 - inter_y1).max(0.0);
        let inter_area = inter_width * inter_height;

        let box_a_area = w1 * h1;
        let box_b_area = w2 * h2;
        let union_area = box_a_area + box_b_area - inter_area;

        if union_area > 0.0 {
            inter_area / union_area
        } else {
            0.0
        }
    }
}

impl ObjectDetector for OnnxObjectDetector {
    /// Detect objects in the given image and return a list of detections.
    fn detect(
        &mut self,
        image: &image::DynamicImage,
    ) -> FrameResult<Vec<Detection>> {
        self.detect(image)
    }
}
