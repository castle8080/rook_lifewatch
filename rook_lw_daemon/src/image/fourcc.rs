

/// Pack 4 ASCII bytes into a Linux/V4L2-style FourCC stored in a `u32`.
///
/// This uses little-endian packing: `a | (b<<8) | (c<<16) | (d<<24)`.
pub const fn fourcc(a: u8, b: u8, c: u8, d: u8) -> u32 {
    u32::from_le_bytes([a, b, c, d])
}

// Common pixel formats
pub const FOURCC_MJPG: u32 = fourcc(b'M', b'J', b'P', b'G');
pub const FOURCC_YUYV: u32 = fourcc(b'Y', b'U', b'Y', b'V');
pub const FOURCC_NV12: u32 = fourcc(b'N', b'V', b'1', b'2');
pub const FOURCC_YU12: u32 = fourcc(b'Y', b'U', b'1', b'2'); // I420
pub const FOURCC_RGB3: u32 = fourcc(b'R', b'G', b'B', b'3'); // RGB24
pub const FOURCC_BGR3: u32 = fourcc(b'B', b'G', b'R', b'3'); // BGR24

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
        .iter()
        .copied()
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
