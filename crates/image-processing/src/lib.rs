//! Image processing, orientation transforms, color management, and pixel buffer operations.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod color;
pub mod grid;
pub mod orientation;
pub mod pixels;

pub use color::{convert_pixel_format, flatten_alpha, merge_alpha_channel, ColorProfileInfo};
pub use grid::assemble_grid;
pub use orientation::{apply_orientation, ExifOrientation, Orientation};
pub use pixels::PixelBuffer;
