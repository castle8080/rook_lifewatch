
use crate::RookLWResult;

pub trait Frame {
    fn get_plane_count(&self) -> RookLWResult<usize>;
    fn get_plane_data(&self, plane_index: usize) -> RookLWResult<&[u8]>;
    fn get_pixel_format(&self) -> RookLWResult<u32>;
    fn get_width(&self) -> RookLWResult<usize>;
    fn get_height(&self) -> RookLWResult<usize>;
}
