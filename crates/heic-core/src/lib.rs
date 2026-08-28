//! Core types, safety limit enforcement, and unified error models for @valen/heic.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod limits;
pub mod types;

pub use error::{HeicError, HeicResult};
pub use limits::Limits;
pub use types::{ColorSpace, ImageDimensions, ImageFormat, PixelFormat};
