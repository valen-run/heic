//! Image processing, orientation transforms, color management, and pixel buffer operations.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod color;
pub mod orientation;
pub mod pixels;

pub use color::ColorProfileInfo;
pub use orientation::ExifOrientation;
pub use pixels::PixelBuffer;
