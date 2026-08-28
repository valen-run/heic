//! HEIC/HEVC decoding traits and pipeline abstractions.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use valen_heic_core::{HeicError, HeicResult, Limits, PixelFormat};
use valen_image_processing::PixelBuffer;

/// Options for configuring image decoding.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DecodeOptions {
    /// Desired pixel format for output buffer.
    pub target_format: Option<PixelFormat>,
    /// Whether to automatically apply EXIF orientation transforms.
    pub apply_orientation: bool,
    /// Resource and safety limits for this decode operation.
    pub limits: Limits,
}

/// Abstract decoder interface for HEIC/HEVC image items.
pub trait HeicDecoder {
    /// Decodes a raw compressed bitstream payload into an uncompressed pixel buffer.
    fn decode_item(&self, payload: &[u8], options: &DecodeOptions) -> HeicResult<PixelBuffer>;
}

/// Placeholder decoder implementation for architecture verification.
#[derive(Debug, Default)]
pub struct PlaceholderDecoder;

impl HeicDecoder for PlaceholderDecoder {
    fn decode_item(&self, _payload: &[u8], options: &DecodeOptions) -> HeicResult<PixelBuffer> {
        options.limits.check_memory_size(0)?;
        Err(HeicError::UnsupportedFeature(
            "HEIC bitstream decoder is under development and not yet implemented".to_string(),
        ))
    }
}
