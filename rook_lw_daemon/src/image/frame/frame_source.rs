use crate::RookLWResult;

use super::Frame;

pub trait FrameSource {

    fn list_sources(&self) -> RookLWResult<Vec<String>>;

    fn set_source(&self, source: &str, required_buffer_count: u32) -> RookLWResult<()>;

    fn get_camera_detail(&self) -> RookLWResult<String>;

    fn start(&self) -> RookLWResult<()>;
    
    fn stop(&self) -> RookLWResult<()>;

    /// Returns the next frame.
    ///
    /// The returned frame is constrained to live no longer than the borrow of
    /// this `FrameSource` reference (i.e. it cannot outlive the `FrameSource`
    /// instance it came from).
    ///
    /// `&self` (shared borrow) is used specifically so you can hold multiple
    /// frames at once and still acquire subsequent frames. Implementations that
    /// need to mutate internal state should use interior mutability.
    fn next_frame(&self) -> RookLWResult<Box<dyn Frame + '_>>;

    fn get_pixel_format(&self) -> RookLWResult<u32>;
    fn get_width(&self) -> RookLWResult<usize>;
    fn get_height(&self) -> RookLWResult<usize>;
}
