use rook_lw_daemon::{RookLWResult, RookLWError};
use rook_lw_daemon::events::ImageProcessingEvent;
use rook_lw_daemon::image::object_detection::ObjectDetector;
use rook_lw_daemon::image::object_detection::opencv_object_detector::OpenCVObjectDetector;
use rook_lw_daemon::image::object_detection::onnx_object_detector::OnnxObjectDetector;
use rook_lw_daemon::image::frame::FrameSource;
use rook_lw_daemon::image::frame_source_factory::FrameSourceFactory;
use rook_lw_daemon::image::fourcc::fourcc_to_string;
use rook_lw_daemon::image::motion::motion_detector::{YPlaneMotionDetector, YPlaneRollingZMotionDetector, YPlaneBoxedAverageMotionDetector};
use rook_lw_daemon::tasks::motion_watcher::MotionWatcher;
use rook_lw_daemon::tasks::image_storer::ImageStorer;
use rook_lw_daemon::tasks::image_detector::ImageDetector;

use rook_lw_image_repo::image_info::{ImageInfoRepository, ImageInfoRepositorySqlite};

use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use std::time::Duration;

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stdout)
        .with_ansi(false)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
    			"%Y-%m-%dT%H:%M:%S%.3f%:z".to_owned(),
    		))
        .with_thread_ids(true)
        .compact()
        .init();
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

fn run_daemon() -> RookLWResult<()> {
    init_tracing();

    let frame_source = create_frame_source()?;

    // MotionWatcher produces CaptureEvents; a separate worker receives and processes them.
    // Bounded provides backpressure so we don't buffer unbounded image data.
    let (motion_detected_tx, motion_detected_rx) = crossbeam_channel::bounded::<ImageProcessingEvent>(64);

    // ImageDetector produces ImageProcessingEvents; ImageStorer receives and processes them.
    let (object_detected_tx, object_detected_rx) = crossbeam_channel::bounded::<rook_lw_daemon::events::ImageProcessingEvent>(64);

    let mut mw = MotionWatcher::new(
        frame_source,
        Duration::from_millis(200), // motion detect interval
        20,     // motion watch count
        create_motion_detector()?,
        5,     // capture count 
        Duration::from_millis(200), // capture interval
        Duration::from_millis(500),    // round interval
    )
    .with_sender(motion_detected_tx);

    // Job that performs object detection on images.
    let object_detector = create_object_detector()?;
    let mut image_detector = ImageDetector::new(
        motion_detected_rx,
        object_detector,
    ).with_sender(object_detected_tx);   

    // Job that stores images to disk.
    let image_info_repository = create_image_info_repository()?;
    let mut image_storer = ImageStorer::new(
        "var/images".to_owned(),
        image_info_repository,
        object_detected_rx,
    );

    let handles = vec![
        std::thread::spawn(move || image_storer.run()),
        std::thread::spawn(move || image_detector.run()),
        std::thread::spawn(move || mw.run()),
    ];

    for handle in handles {
        match handle.join() {
            Ok(result) => {
                if let Err(e) = result {
                    error!(error = %e, "Task failed");
                }
            },
            Err(e) => {
                error!(error = ?e, "Task panicked");
            }
        }
    }

	info!("Complete");
    Ok(())
}

fn main() -> RookLWResult<()> {
    run_daemon()
    //run_test_detection()
}
