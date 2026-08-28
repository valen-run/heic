//! PNG encoding options and placeholder encoder.

use crate::ImageEncoder;
use valen_heic_core::{HeicError, HeicResult};
use valen_image_processing::PixelBuffer;

/// Encoding options for PNG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PngOptions {
    /// Compression level from 0 to 9.
    pub compression_level: u8,
}

/// Placeholder PNG encoder.
#[derive(Debug, Default)]
pub struct PngEncoder;

impl ImageEncoder for PngEncoder {
    type Options = PngOptions;

    fn encode(&self, _buffer: &PixelBuffer, _options: &Self::Options) -> HeicResult<Vec<u8>> {
        Err(HeicError::EncodeError(
            "PNG encoder is under development".to_string(),
        ))
    }
}
