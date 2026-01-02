use rook_life_watch::core::frame::FrameSource;
use rook_life_watch::core::pipeline::FrameProcessor;
use rook_life_watch::error::RookLWResult;
use rook_life_watch::pipeline::simple_motion::SimpleFrameDiffer;
use rook_life_watch::platform::desktop::StubDesktopFrameSource;

fn main() -> RookLWResult<()> {
    let mut frame_source = StubDesktopFrameSource;
    let mut processor = SimpleFrameDiffer;

    let frame = frame_source.next_frame()?;
    let events = processor.process_frame(frame)?;

    for event in events {
        println!("motion event: {:?}", event);
    }

    Ok(())
}
