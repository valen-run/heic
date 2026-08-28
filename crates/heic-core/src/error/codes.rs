//! Stable error code strings for API and WebAssembly mapping.

use super::HeicError;

impl HeicError {
    /// Returns a stable, machine-readable string error code for JS/WASM error mapping.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::InvalidContainer(_) => "INVALID_CONTAINER",
            Self::MalformedInput(_) => "MALFORMED_INPUT",
            Self::UnsupportedFormat(_) => "UNSUPPORTED_FORMAT",
            Self::UnsupportedBrand(_) => "UNSUPPORTED_BRAND",
            Self::UnsupportedCodec(_) => "UNSUPPORTED_CODEC",
            Self::UnsupportedSequence(_) => "UNSUPPORTED_SEQUENCE",
            Self::UnsupportedFeature(_) => "UNSUPPORTED_FEATURE",
            Self::LimitExceeded(_) => "LIMIT_EXCEEDED",
            Self::LimitInputBytes { .. } => "LIMIT_INPUT_BYTES",
            Self::LimitDimensions { .. } => "LIMIT_DIMENSIONS",
            Self::LimitPixels { .. } => "LIMIT_PIXELS",
            Self::PixelLimitExceeded { .. } => "PIXEL_LIMIT_EXCEEDED",
            Self::LimitMemory { .. } => "LIMIT_MEMORY",
            Self::DecodeError(_) => "DECODE_ERROR",
            Self::DecodeFailed(_) => "DECODE_FAILED",
            Self::EncodeError(_) => "ENCODE_ERROR",
            Self::EncodeFailed(_) => "ENCODE_FAILED",
            Self::Aborted => "OPERATION_ABORTED",
            Self::Timeout => "TIMEOUT",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}
