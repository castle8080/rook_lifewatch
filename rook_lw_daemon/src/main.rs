use std::fs;

use rook_lw_daemon::error::RookLWResult;
use rook_lw_daemon::image::frame_source_factory::FrameSourceFactory;
use rook_lw_daemon::image::fourcc::fourcc_to_string;
use rook_lw_daemon::tasks::motion_watcher::MotionWatcher;

fn main() -> RookLWResult<()> {

    let image_dump_dir = "var/images";

    fs::create_dir_all(image_dump_dir)?;

    // Print available frame sources at compile time
    println!("Available frame sources: {:?}", FrameSourceFactory::available_sources());

    let mut frame_source = FrameSourceFactory::create()?;

    let sources = frame_source.list_sources()?;

    for (i, source) in sources.iter().enumerate() {
        println!("Source {}: {}", i, source);
    }

    if sources.is_empty() {
        return Err(rook_lw_daemon::error::RookLWError::Camera("No available frame sources found".to_owned()));
    }
    else {
        println!("Using source: {}", sources[0]);
        frame_source.set_source(&sources[0])?;
    }

    let pixel_format = fourcc_to_string(frame_source.get_pixel_format()?);
    println!("Pixel format: {}", pixel_format);

    if pixel_format != "MJPG" {
        return Err(rook_lw_daemon::error::RookLWError::Camera(format!("Expected MJPG pixel format, got {}", pixel_format)));
    }

    println!("Frame dimensions: {}x{}", frame_source.get_width()?, frame_source.get_height()?);

    let mut mw = MotionWatcher::new(
        frame_source,
        std::time::Duration::from_millis(100), // motion detect interval
        10,     // motion watch count
        0.025,  // motion threshold
        10,     // capture count 
        std::time::Duration::from_millis(200), // capture interval
        std::time::Duration::from_secs(5),    // round interval
    );
    mw.run()?;

    println!("Complete...");
    Ok(())
}
