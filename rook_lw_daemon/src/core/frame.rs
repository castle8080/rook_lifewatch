
#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("capture error: {0}")]
    Capture(String),
    #[error("no frame source implementation available")]
    NoImplementationAvailable,
    #[error("failed to initialize frame source: {0}")]
    InitializationFailed(String),
    #[error("processing error: {0}")]
    ProcessingError(String),
}

pub type FrameResult<T> = Result<T, FrameError>;

pub trait Frame {
    fn get_plane_count(&self) -> FrameResult<usize>;
    fn get_plane_data(&self, plane_index: usize) -> FrameResult<&[u8]>;
}

pub trait FrameSource {

    fn list_sources(&mut self) -> FrameResult<Vec<String>>;

    fn set_source(&mut self, source: &str) -> FrameResult<()>;

    fn start(&mut self) -> FrameResult<()>;

    fn stop(&mut self) -> FrameResult<()>;

    /// Returns the next frame.
    ///
    /// The returned frame is constrained to live no longer than the borrow of
    /// this `FrameSource` reference (i.e. it cannot outlive the `FrameSource`
    /// instance it came from).
    ///
    /// `&self` (shared borrow) is used specifically so you can hold multiple
    /// frames at once and still acquire subsequent frames. Implementations that
    /// need to mutate internal state should use interior mutability.
    fn next_frame(&self) -> FrameResult<Box<dyn Frame + '_>>;

    fn get_pixel_format(&self) -> FrameResult<u32>;
}

/// Converts a FourCC stored in a `u32` into a 4-character `String`.
///
/// This assumes the common Linux convention where a FourCC is packed as:
///
/// ```text
/// code = a | (b << 8) | (c << 16) | (d << 24)
/// ```
///
/// (i.e. the byte sequence is interpreted as little-endian).
///
/// Non-printable bytes are replaced with `?` so the returned string is always
/// exactly 4 characters long.
pub fn fourcc_to_string(code: u32) -> String {
    let bytes = code.to_le_bytes();
    bytes
        .into_iter()
        .map(|b| {
            if b.is_ascii_graphic() || b == b' ' {
                b as char
            } else {
                '?'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::fourcc_to_string;

    #[test]
    fn fourcc_to_string_basic() {
        let code = u32::from_le_bytes(*b"YUYV");
        assert_eq!(fourcc_to_string(code), "YUYV");
    }

    #[test]
    fn fourcc_to_string_non_printable_replaced() {
        let code = u32::from_le_bytes([0, b'A', 0x7F, b' ']);
        assert_eq!(fourcc_to_string(code), "?A? ");
    }
}
