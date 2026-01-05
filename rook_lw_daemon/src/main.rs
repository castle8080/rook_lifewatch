use rook_life_watch::core::pipeline::FrameProcessor;
use rook_life_watch::error::RookLWResult;
use rook_life_watch::pipeline::simple_motion::SimpleFrameDiffer;
use rook_life_watch::implementation::factory::FrameSourceFactory;

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
        return Err(rook_life_watch::error::RookLWError::Camera("No available frame sources found".to_owned()));
    }
    else {
        println!("Using source: {}", sources[0]);
        frame_source.set_source(&sources[0])?;
    }

    // Alternative: specify preference order
    // let mut frame_source = FrameSourceFactory::create_with_preference(
    //     FrameSourcePreference::PreferOpenCV
    // ).expect("Failed to create frame source");

    let mut processor = SimpleFrameDiffer;

    let frame = frame_source.next_frame()?;
    let events = processor.process_frame(frame)?;

    for event in events {
        println!("motion event: {:?}", event);
    }

    Ok(())
}
