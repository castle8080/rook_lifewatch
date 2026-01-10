use rook_lw_daemon::error::RookLWResult;
use rook_lw_daemon::image::frame_source_factory::FrameSourceFactory;
use rook_lw_daemon::image::fourcc::fourcc_to_string;
use rook_lw_daemon::tasks::motion_watcher::MotionWatcher;

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

fn main() -> RookLWResult<()> {
    init_tracing();

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

    let mut mw = MotionWatcher::new(
        frame_source,
        std::time::Duration::from_millis(100), // motion detect interval
        10,     // motion watch count
        0.03,  // motion threshold
        10,     // capture count 
        std::time::Duration::from_millis(200), // capture interval
        std::time::Duration::from_secs(5),    // round interval
    );
    mw.run()?;

	info!("Complete");
    Ok(())
}
