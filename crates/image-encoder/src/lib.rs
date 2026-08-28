//! Image encoding interfaces and format-specific encoder implementations.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod jpeg;
pub mod png;
pub mod webp;

pub use jpeg::{JpegEncoder, JpegOptions};
pub use png::{PngEncoder, PngOptions};
pub use webp::{WebpEncoder, WebpOptions};

use valen_heic_core::HeicResult;
use valen_image_processing::PixelBuffer;

/// Generic image encoder interface.
pub trait ImageEncoder {
    /// Encoder specific options type.
    type Options;

    /// Encodes an uncompressed pixel buffer to encoded bytes.
    fn encode(&self, buffer: &PixelBuffer, options: &Self::Options) -> HeicResult<Vec<u8>>;
}
