//! Error definitions for the HEIC/HEIF processing pipeline.

use thiserror::Error;

/// Result alias using [`HeicError`].
pub type HeicResult<T> = Result<T, HeicError>;

/// Represents errors that can occur during HEIC/HEIF inspection, parsing, decoding, or encoding.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum HeicError {
    /// The input format or brand is not supported.
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    /// The container structure is invalid or corrupt.
    #[error("Invalid HEIF/HEIC container: {0}")]
    InvalidContainer(String),

    /// Malformed input data or unexpected EOF encountered.
    #[error("Malformed input: {0}")]
    MalformedInput(String),

    /// File size or image dimensions exceed configured resource limits.
    #[error("Resource limit exceeded: {0}")]
    LimitExceeded(String),

    /// Image dimensions or pixel count exceeds allowed threshold.
    #[error("Pixel limit exceeded: count={count}, max={max}")]
    PixelLimitExceeded {
        /// Actual or calculated pixel count
        count: u64,
        /// Configured maximum pixel count
        max: u64,
    },

    /// Decoding failed.
    #[error("Decoding failure: {0}")]
    DecodeError(String),

    /// Encoding to the target format failed.
    #[error("Encoding failure: {0}")]
    EncodeError(String),

    /// The operation was aborted/cancelled by the caller.
    #[error("Operation aborted")]
    Aborted,

    /// A feature in the image is not supported (e.g., unsupported bit depth or compression).
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl HeicError {
    /// Returns a machine-readable error code corresponding to JavaScript error discriminants.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::UnsupportedFormat(_) => "UNSUPPORTED_FORMAT",
            Self::InvalidContainer(_) => "INVALID_CONTAINER",
            Self::MalformedInput(_) => "MALFORMED_INPUT",
            Self::LimitExceeded(_) => "LIMIT_EXCEEDED",
            Self::PixelLimitExceeded { .. } => "PIXEL_LIMIT_EXCEEDED",
            Self::DecodeError(_) => "DECODE_ERROR",
            Self::EncodeError(_) => "ENCODE_ERROR",
            Self::Aborted => "OPERATION_ABORTED",
            Self::UnsupportedFeature(_) => "UNSUPPORTED_FEATURE",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}
