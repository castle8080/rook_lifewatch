use crate::app::{App, AppConfiguration};
use crate::tasks::image_capturer::ImageCapturer;
use crate::{RookLWResult, RookLWError};
use crate::image::object_detection::ObjectDetector;
use crate::image::object_detection::OpenCVObjectDetector;
use crate::image::object_detection::OnnxObjectDetector;
use crate::image::frame::FrameSource;
use crate::image::frame::FrameSourceFactory;

use std::sync::Arc;
use crate::image::fourcc::fourcc_to_string;
use crate::image::motion::{YPlaneMotionDetector, YPlaneRollingZMotionDetector, YPlaneBoxedAverageMotionDetector, YPlaneMotionPercentileDetector};
use crate::tasks::image_diff_motion_watcher::ImageDiffMotionWatcher;
use crate::tasks::radar_motion_watcher::RadarMotionWatcher;
use crate::tasks::motion_watcher::MotionWatcher;
use crate::tasks::image_storer::ImageStorer;
use crate::tasks::image_detector::ImageDetector;

use rook_lw_image_repo::sqlite::create_pool;
use rook_lw_image_repo::image_info::{ImageInfoRepository, ImageInfoRepositorySqlite};
use rook_lw_image_repo::image_store::{ImageStoreRepository, ImageStoreRepositoryFile};

use tracing::{error, info};

use std::time::Duration;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub fn create_app() -> RookLWResult<App> {

    // create app configuration
    let app_config = AppConfiguration::load("config/rook_lw_daemon.toml")?;
    info!(app_configuration = ?app_config, "App configuration loaded");

    // Create SQLite connection pool
    let db_pool = create_sqlite_pool(&app_config)?;

    let frame_source = create_frame_source(&app_config)?;

    // Create the motion watcher.
    let mw = create_motion_watcher(&app_config, frame_source)?;

    // Job that performs object detection on images.
    let image_detector = create_image_detector(&app_config)?;

    // Job that stores images to disk.
    let image_info_repository = create_image_info_repository(db_pool)?;
    let image_store_repository = create_image_store_repository(&app_config)?;

    let image_storer = create_image_storer(
        image_store_repository,
        image_info_repository,
    )?;

    let app = App::new(
        mw,
        image_storer,
        image_detector,
    );

    Ok(app)
}

fn create_image_storer(
    image_store_repository: Box<dyn ImageStoreRepository>,
    image_info_repository: Box<dyn ImageInfoRepository>
    ) -> RookLWResult<ImageStorer>
{
    Ok(ImageStorer::new(
        image_store_repository,
        image_info_repository,
    ))
}

fn create_image_store_repository(app_config: &AppConfiguration) -> RookLWResult<Box<dyn ImageStoreRepository>> {
    let repo = ImageStoreRepositoryFile::new(
        (&app_config.image_directory).into()
    )?;

    Ok(Box::new(repo))
}

fn create_image_detector(app_config: &AppConfiguration) -> RookLWResult<ImageDetector> {
    let object_detector = create_object_detector(app_config)?;
    Ok(ImageDetector::new(
        object_detector,
    ))
}

fn creat_image_capturer(app_config: &AppConfiguration, frame_source: Arc<Box<dyn FrameSource + Send + Sync>>) -> ImageCapturer {
    ImageCapturer::new(
        frame_source,
        app_config.image_capturer_capture_count,
        Duration::from_millis(app_config.image_capturer_capture_interval_ms),
    )
}

fn create_motion_watcher(app_config: &AppConfiguration, frame_source: Arc<Box<dyn FrameSource + Send + Sync>>) -> RookLWResult<Box<dyn MotionWatcher>> {
    let image_capturer = creat_image_capturer(app_config, frame_source.clone());
    
    match app_config.motion_watcher_type.as_str() {
        "radar" => {
            let watcher = RadarMotionWatcher::new(
                app_config.radar_gpio_chip_path.clone(),
                app_config.radar_gpio_pin,
                image_capturer,
            );
            Ok(Box::new(watcher))
        },
        _ => {
            let watcher = ImageDiffMotionWatcher::new(
                frame_source.clone(),
                Duration::from_millis(app_config.motion_watcher_round_interval_ms), // motion detect interval
                app_config.motion_watcher_count,     // motion watch count
                create_motion_detector(app_config)?,
                image_capturer,
                Duration::from_millis(app_config.motion_watcher_round_interval_ms),    // round interval
            );
            Ok(Box::new(watcher))
        }
    }
}

fn create_frame_source(app_config: &AppConfiguration) -> RookLWResult<Arc<Box<dyn FrameSource + Send + Sync>>> {
   // Print available frame sources at compile time
    info!(available_sources = ?FrameSourceFactory::available_sources(), "Available frame sources");

    // Create frame source based on configuration
    let frame_source = match &app_config.frame_source {
        Some(frame_source_name) => {
            info!(source_name = %frame_source_name, "Creating specified frame source");
            FrameSourceFactory::try_create(frame_source_name)?
        },
        None => {
            info!("Creating default frame source");
            FrameSourceFactory::create()?
        }
    };

    // Choose the camera.
    match &app_config.camera_source {
        Some(camera_source_name) => {
            info!(camera_source = %camera_source_name, "Setting specified camera source");
            frame_source.set_source(camera_source_name, 2)?; // use 2 buffers
        },
        None => {
            let sources = frame_source.list_sources()?;
            if sources.is_empty() {
                error!("No available frame sources found");
                return Err(RookLWError::Camera("No available frame sources found".to_owned()));
            }
            else {
                info!(source = %sources[0], "Using default camera source");
                frame_source.set_source(&sources[0], 2)?; // use 2 buffers
            }
        }
    }

    let pixel_format = fourcc_to_string(frame_source.get_pixel_format()?);
	info!(pixel_format = %pixel_format, "Camera pixel format");
	info!(width = frame_source.get_width()?, height = frame_source.get_height()?, "Frame dimensions");
    
    info!("Frame source created successfully. Details:\n{}",
        frame_source.get_camera_detail().unwrap_or_else(|e| e.to_string())
    );

    Ok(Arc::new(frame_source))
}

fn create_motion_detector(app_config: &AppConfiguration) -> RookLWResult<Box<dyn YPlaneMotionDetector>> {
    
    fn add_rolling_z_if_enabled<T>(app_config: &AppConfiguration, base_detector: T) -> RookLWResult<Box<dyn YPlaneMotionDetector>>
        where T: YPlaneMotionDetector + 'static
    {
        if app_config.use_yplane_rolling_z {
            info!("Using Y-Plane Rolling Z motion detector");

            let motion_detector = YPlaneRollingZMotionDetector::new(
                base_detector, 
                app_config.yplane_rolling_z_alpha, 
                app_config.yplane_rolling_z_threshold
            )?;

            Ok(Box::new(motion_detector))
        }
        else {
            Ok(Box::new(base_detector))
        }
    }

    match app_config.motion_detector_type.as_str() {
        "yplane_motion_percentile" => {
            add_rolling_z_if_enabled(app_config, YPlaneMotionPercentileDetector::new(
                app_config.yplane_motion_percentile,
                app_config.yplane_motion_percentile_threshold,
            ))
        },
        "yplane_boxed_average" => {
            add_rolling_z_if_enabled(app_config, YPlaneBoxedAverageMotionDetector::new(
                app_config.yplane_boxed_average_motion_detector_box_size, 
                app_config.yplane_boxed_average_motion_detector_percentile,
                app_config.yplane_boxed_average_motion_detector_threshold,
            ))
        },
        other => Err(RookLWError::Initialization(format!(
            "Unknown motion detector type: {}",
            other
        ))),
    }
}

fn create_object_detector(app_config: &AppConfiguration) -> RookLWResult<Box<dyn ObjectDetector>> {
    match app_config.object_detector_type.as_str() {
        "onnx" => Ok(Box::new(create_onnx_object_detector(app_config)?)),
        "opencv" => Ok(Box::new(create_opencv_object_detector(app_config)?)),
        other => Err(RookLWError::Initialization(format!(
            "Unknown object detector type: {}",
            other
        ))),
    }
}

fn create_onnx_object_detector(app_config: &AppConfiguration) -> RookLWResult<OnnxObjectDetector> {
    let object_detector = OnnxObjectDetector::new(
        app_config.onnx_model_path.as_str(),
        app_config.onnx_model_names_path.as_str(),
        app_config.onnx_model_confidence_threshold,  // YOLO confidence threshold
    )?;

    Ok(object_detector)
}

fn create_opencv_object_detector(app_config: &AppConfiguration) -> RookLWResult<OpenCVObjectDetector> {
    let object_detector = OpenCVObjectDetector::new(
        app_config.opencv_model_config_path.as_str(),
        app_config.opencv_model_weights_path.as_str(),
        app_config.opencv_model_names_path.as_str(),
        app_config.opencv_model_confidence_threshold  // YOLO confidence threshold
    )?;

    Ok(object_detector)
}

fn create_sqlite_pool(app_config: &AppConfiguration) -> RookLWResult<Pool<SqliteConnectionManager>> {
    let pool = create_pool(&app_config.database_path)?;
    Ok(pool)
}

fn create_image_info_repository(pool: Pool<SqliteConnectionManager>) -> RookLWResult<Box<dyn ImageInfoRepository>> {
    let repo = ImageInfoRepositorySqlite::new(
        pool
    )?;

    Ok(Box::new(repo))
}


