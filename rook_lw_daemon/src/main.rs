use rook_life_watch::core::pipeline::FrameProcessor;
use rook_life_watch::error::RookLWResult;
use rook_life_watch::pipeline::simple_motion::SimpleFrameDiffer;
use rook_life_watch::implementation::factory::FrameSourceFactory;

fn main() -> RookLWResult<()> {
    // Print available frame sources at compile time
    println!("Available frame sources: {:?}", FrameSourceFactory::available_sources());

    // Create frame source using factory with default preference
    let mut frame_source = FrameSourceFactory::create()
        .expect("Failed to create frame source");

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
