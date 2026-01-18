use self_cell::self_cell;

use crate::RookLWResult;
use crate::image::frame::Frame;
use crate::image::yplane;
use crate::image::yplane::YPlane;

self_cell!(
    pub struct FrameSlot<'fs> {
        owner: Box<dyn Frame + 'fs>,

        #[covariant]
        dependent: YPlane,
    }
);

impl<'fs> FrameSlot<'fs> {
    /// Construct a slot by owning the frame and caching its Y plane.
    ///
    /// For YUYV this will be a borrowed view into the frame data (no copy).
    /// For MJPG this will decode and own the luma buffer (one decode per frame).
    pub fn from_frame(frame: Box<dyn Frame + 'fs>) -> RookLWResult<Self> {
        // self_cell supports fallible dependent construction via `try_new`.
        FrameSlot::try_new(frame, |f| yplane::YPlane::from_frame(&**f))
    }

    pub fn yplane(&self) -> &yplane::YPlane<'_> {
        self.borrow_dependent()
    }

    pub fn frame(&self) -> &dyn Frame {
        &**self.borrow_owner()
    }
}
