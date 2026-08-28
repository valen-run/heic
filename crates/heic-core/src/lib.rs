//! Core types, safety limit enforcement, and unified error models for @valen-run/heic.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod limits;
pub mod types;

pub use error::{HeicError, HeicResult};
pub use limits::{
    checked_mul, checked_mul_usize, Limits, DEFAULT_MAX_DIMENSION, DEFAULT_MAX_FILE_SIZE,
    DEFAULT_MAX_ITEM_COUNT, DEFAULT_MAX_MEMORY_BYTES, DEFAULT_MAX_PIXEL_COUNT,
    DEFAULT_MAX_TILE_COUNT,
};
pub use types::{ColorSpace, ImageDimensions, ImageFormat, Orientation, OutputFormat, PixelFormat};
