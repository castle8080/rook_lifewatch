use std::thread::sleep;

use rook_lw_daemon::error::RookLWResult;
use rook_lw_daemon::implementation::factory::FrameSourceFactory;

fn main() -> RookLWResult<()> {
    // Print available frame sources at compile time
    println!("Available frame sources: {:?}", FrameSourceFactory::available_sources());

    // Create frame source using factory with default preference
    //let camera = "rtsp://192.168.1.21:8554/";
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

    println!("Starting frame source...");

    frame_source.start()?;

    sleep(std::time::Duration::from_secs(1));

    let frame1 = frame_source.next_frame()?;
    println!("Acquired first frame...");
    sleep(std::time::Duration::from_secs(1));

    let frame2 = frame_source.next_frame()?;
    println!("Acquired second frame...");
    sleep(std::time::Duration::from_secs(1));

    println!("Complete...");
    Ok(())
}
