use rook_lw_daemon::error::RookLWResult;
use rook_lw_daemon::image::frame_source_factory::FrameSourceFactory;
use rook_lw_daemon::image::fourcc::fourcc_to_string;
use rook_lw_daemon::image::motion::motion_detector::{YPlaneMotionDetector, YPlaneMotionPercentileDetector, YPlaneRollingZMotionDetector};
use rook_lw_daemon::tasks::motion_watcher::MotionWatcher;
use rook_lw_daemon::tasks::image_storer::ImageStorer;
use rook_lw_daemon::events::capture_event::CaptureEvent;

use tracing::{error, info};
use tracing_subscriber::EnvFilter;

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

fn create_frame_source() -> RookLWResult<Box<dyn rook_lw_daemon::image::frame::FrameSource + Send>> {
   // Print available frame sources at compile time
    info!(available_sources = ?FrameSourceFactory::available_sources(), "Available frame sources");

    let mut frame_source = FrameSourceFactory::create()?;

    let sources = frame_source.list_sources()?;

    for (i, source) in sources.iter().enumerate() {
		info!(index = i, source = %source, "Camera source");
    }

    if sources.is_empty() {
		error!("No available frame sources found");
        return Err(rook_lw_daemon::error::RookLWError::Camera("No available frame sources found".to_owned()));
    }
    else {
		info!(source = %sources[0], "Using camera source");
        frame_source.set_source(&sources[0])?;
    }

    let pixel_format = fourcc_to_string(frame_source.get_pixel_format()?);
	info!(pixel_format = %pixel_format, "Camera pixel format");

    if pixel_format != "MJPG" {
		error!(pixel_format = %pixel_format, "Unexpected pixel format (expected MJPG)");
        return Err(rook_lw_daemon::error::RookLWError::Camera(format!("Expected MJPG pixel format, got {}", pixel_format)));
    }

	info!(width = frame_source.get_width()?, height = frame_source.get_height()?, "Frame dimensions");
    
    Ok(frame_source)
}

fn create_motion_detector() -> RookLWResult<Box<dyn YPlaneMotionDetector>> {
    let base_motion_detector = YPlaneMotionPercentileDetector::new(0.95, 1.0);
    let motion_detector = YPlaneRollingZMotionDetector::new(base_motion_detector, 0.05, 2.0)?;

    Ok(Box::new(motion_detector))
}

fn main() -> RookLWResult<()> {
    init_tracing();

    let frame_source = create_frame_source()?;

    // MotionWatcher produces CaptureEvents; a separate worker receives and processes them.
    // Bounded provides backpressure so we don't buffer unbounded image data.
    let (capture_event_tx, capture_event_rx) = crossbeam_channel::bounded::<CaptureEvent>(64);

    let mut image_storer = ImageStorer::new(
        "var/images".to_owned(),
        capture_event_rx,
    );

    let mut mw = MotionWatcher::new(
        frame_source,
        capture_event_tx,
        std::time::Duration::from_millis(100), // motion detect interval
        10,     // motion watch count
        create_motion_detector()?,
        5,     // capture count 
        std::time::Duration::from_millis(200), // capture interval
        std::time::Duration::from_secs(2),    // round interval
    );

    let image_storer_handle = std::thread::spawn(move || {
        image_storer.run()
    });

    let mw_handle = std::thread::spawn(move || {
        mw.run()
    });

    match image_storer_handle.join() {
        Ok(result) => {
            if let Err(e) = result {
                error!(error = %e, "Image storer task failed");
            }
        },
        Err(e) => {
            error!(error = ?e, "Image storer task panicked");
        }
    }

    match mw_handle.join() {
        Ok(result) => {
            if let Err(e) = result {
                error!(error = %e, "Motion watcher task failed");
            }
        },
        Err(e) => {
            error!(error = ?e, "Motion watcher task panicked");
        }
    }

	info!("Complete");
    Ok(())
}
