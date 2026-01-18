use crate::app::App;
use crate::{RookLWResult, RookLWError};
use crate::image::object_detection::ObjectDetector;
use crate::image::object_detection::OpenCVObjectDetector;
use crate::image::object_detection::OnnxObjectDetector;
use crate::image::frame::FrameSource;
use crate::image::frame::FrameSourceFactory;
use crate::image::fourcc::fourcc_to_string;
use crate::image::motion::{YPlaneMotionDetector, YPlaneRollingZMotionDetector, YPlaneBoxedAverageMotionDetector};
use crate::tasks::motion_watcher::MotionWatcher;
use crate::tasks::image_storer::ImageStorer;
use crate::tasks::image_detector::ImageDetector;

use rook_lw_image_repo::image_info::{ImageInfoRepository, ImageInfoRepositorySqlite};

use tracing::{error, info};

use std::time::Duration;

pub fn create_app() -> RookLWResult<App> {

    let frame_source = create_frame_source()?;

    let mw = MotionWatcher::new(
        frame_source,
        Duration::from_millis(200), // motion detect interval
        20,     // motion watch count
        create_motion_detector()?,
        5,     // capture count 
        Duration::from_millis(200), // capture interval
        Duration::from_millis(500),    // round interval
    );

    // Job that performs object detection on images.
    let object_detector = create_object_detector()?;
    let image_detector = ImageDetector::new(
        object_detector,
    );

    // Job that stores images to disk.
    let image_info_repository = create_image_info_repository()?;
    let image_storer = ImageStorer::new(
        "var/images".to_owned(),
        image_info_repository,
    );

    let app = App::new(
        mw,
        image_storer,
        image_detector,
    );

    Ok(app)
}

fn create_frame_source() -> RookLWResult<Box<dyn FrameSource + Send>> {
   // Print available frame sources at compile time
    info!(available_sources = ?FrameSourceFactory::available_sources(), "Available frame sources");

    let mut frame_source = FrameSourceFactory::create()?;
    let sources = frame_source.list_sources()?;

    for (i, source) in sources.iter().enumerate() {
		info!(index = i, source = %source, "Camera source");
    }

    if sources.is_empty() {
		error!("No available frame sources found");
        return Err(RookLWError::Camera("No available frame sources found".to_owned()));
    }
    else {
		info!(source = %sources[0], "Using camera source");
        frame_source.set_source(&sources[0])?;
    }

    let pixel_format = fourcc_to_string(frame_source.get_pixel_format()?);
	info!(pixel_format = %pixel_format, "Camera pixel format");

    if pixel_format != "MJPG" {
		error!(pixel_format = %pixel_format, "Unexpected pixel format (expected MJPG)");
        return Err(RookLWError::Camera(format!("Expected MJPG pixel format, got {}", pixel_format)));
    }

	info!(width = frame_source.get_width()?, height = frame_source.get_height()?, "Frame dimensions");
    
    Ok(frame_source)
}

fn create_motion_detector() -> RookLWResult<Box<dyn YPlaneMotionDetector>> {
    //let base_motion_detector = YPlaneMotionPercentileDetector::new(0.95, 0.02);
    //let motion_detector = YPlaneRollingZMotionDetector::new(base_motion_detector, 0.05, 2.0)?;

    let base_motion_detector = YPlaneBoxedAverageMotionDetector::new(
        50, 
        0.98,
        0.02
    );

    let motion_detector = YPlaneRollingZMotionDetector::new(
        base_motion_detector, 
        0.05, 
        2.0
    )?;

    Ok(Box::new(motion_detector))
}

fn create_object_detector() -> RookLWResult<Box<dyn ObjectDetector>> {
    //Ok(Box::new(create_onnx_object_detector()?))
    Ok(Box::new(create_opencv_object_detector()?))
}

fn create_onnx_object_detector() -> RookLWResult<OnnxObjectDetector> {
    let object_detector = OnnxObjectDetector::new(
        "models/yolov4-tiny.onnx",
        "models/coco.names",
        0.15  // YOLO confidence threshold
    )?;

    Ok(object_detector)
}

fn create_opencv_object_detector() -> RookLWResult<OpenCVObjectDetector> {
    let object_detector = OpenCVObjectDetector::new(
        "models/yolov4-tiny.cfg",
        "models/yolov4-tiny.weights",
        "models/coco.names",
        0.15  // YOLO confidence threshold
    )?;

    Ok(object_detector)
}

fn create_image_info_repository() -> RookLWResult<Box<dyn ImageInfoRepository>> {
    let repo = ImageInfoRepositorySqlite::new_from_path(
        "var/db/image_info.db"
    )?;

    Ok(Box::new(repo))
}



