//! OpenCV-based frame capture implementation.
//!
//! This module provides `Frame` and `FrameSource` implementations using OpenCV's
//! VideoCapture backend. It supports:
//! - Local camera devices (by numeric ID)
//! - Network video streams (RTSP, HTTP)
//! - Video files
//!
//! # Example
//!
//! ```ignore
//! use rook_lw_daemon::image::opencv::{OpenCvFrameSource, OpenCvFrame};
//! use rook_lw_daemon::image::frame::FrameSource;
//!
//! let mut source = OpenCvFrameSource::new()?;
//! source.set_source("0")?;  // Open camera 0
//! source.start()?;
//!
//! let frame = source.next_frame()?;
//! println!("Frame size: {}x{}", frame.get_width()?, frame.get_height()?);
//! ```

mod opencv_frame;
mod opencv_frame_source;

pub use opencv_frame::OpenCvFrame;
pub use opencv_frame_source::OpenCvFrameSource;
