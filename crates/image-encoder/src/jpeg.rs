//! JPEG encoding options and placeholder encoder.

use crate::ImageEncoder;
use valen_heic_core::{HeicError, HeicResult};
use valen_image_processing::PixelBuffer;

/// Encoding options for JPEG.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JpegOptions {
    /// JPEG quality from 1 to 100 (default 85).
    pub quality: u8,
}

impl Default for JpegOptions {
    fn default() -> Self {
        Self { quality: 85 }
    }
}

/// Placeholder JPEG encoder.
#[derive(Debug, Default)]
pub struct JpegEncoder {
    /// Encoding options.
    pub options: JpegOptions,
}

impl ImageEncoder for JpegEncoder {
    type Options = JpegOptions;

    fn encode(&self, _buffer: &PixelBuffer, _options: &Self::Options) -> HeicResult<Vec<u8>> {
        Err(HeicError::EncodeError(
            "JPEG encoder is under development".to_string(),
        ))
    }
}
