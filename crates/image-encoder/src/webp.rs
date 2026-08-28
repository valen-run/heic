//! WebP encoding options and placeholder encoder.

use crate::ImageEncoder;
use valen_heic_core::{HeicError, HeicResult};
use valen_image_processing::PixelBuffer;

/// Encoding options for WebP.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WebpOptions {
    /// Quality from 0.0 to 100.0.
    pub quality: f32,
    /// Whether to use lossless compression.
    pub lossless: bool,
}

impl Default for WebpOptions {
    fn default() -> Self {
        Self {
            quality: 80.0,
            lossless: false,
        }
    }
}

/// Placeholder WebP encoder.
#[derive(Debug, Default)]
pub struct WebpEncoder;

impl ImageEncoder for WebpEncoder {
    type Options = WebpOptions;

    fn encode(&self, _buffer: &PixelBuffer, _options: &Self::Options) -> HeicResult<Vec<u8>> {
        Err(HeicError::EncodeError(
            "WebP encoder is under development".to_string(),
        ))
    }
}
