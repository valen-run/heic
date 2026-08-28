//! Core error enum definition for the HEIC/HEIF processing pipeline.

use thiserror::Error;

/// Result alias using [`HeicError`].
pub type HeicResult<T> = Result<T, HeicError>;

/// Represents errors that can occur during HEIC/HEIF inspection, parsing, decoding, or encoding.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum HeicError {
    /// The input bytes do not contain a valid HEIF/HEIC container or signature.
    #[error("Invalid input data: {0}")]
    InvalidInput(String),

    /// The container structure (e.g. ISO-BMFF box structure) is invalid or corrupt.
    #[error("Invalid HEIF/HEIC container: {0}")]
    InvalidContainer(String),

    /// Malformed input data or unexpected EOF encountered during parsing.
    #[error("Malformed input: {0}")]
    MalformedInput(String),

    /// The input format is not supported.
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    /// The container brand (e.g. major or compatible brand) is not supported.
    #[error("Unsupported brand: {0}")]
    UnsupportedBrand(String),

    /// The compression codec (e.g., non-HEVC/H.265 bitstream) is not supported.
    #[error("Unsupported codec: {0}")]
    UnsupportedCodec(String),

    /// Image sequence detected when still-image decoding was requested.
    #[error("Unsupported image sequence: {0}")]
    UnsupportedSequence(String),

    /// A specific image feature or profile is not supported.
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    /// Generic resource limit exceeded.
    #[error("Resource limit exceeded: {0}")]
    LimitExceeded(String),

    /// Input byte size exceeds configured maximum budget.
    #[error("Input byte limit exceeded: actual={actual}, max={max}")]
    LimitInputBytes {
        /// Actual input size in bytes.
        actual: u64,
        /// Maximum allowed input size in bytes.
        max: u64,
    },

    /// Image dimensions exceed configured maximum width or height.
    #[error("Dimension limit exceeded: width={width}, height={height}")]
    LimitDimensions {
        /// Actual image width in pixels.
        width: u32,
        /// Actual image height in pixels.
        height: u32,
        /// Configured maximum width if any.
        max_width: Option<u32>,
        /// Configured maximum height if any.
        max_height: Option<u32>,
    },

    /// Total pixel count exceeds configured limit.
    #[error("Pixel limit exceeded: count={count}, max={max}")]
    LimitPixels {
        /// Actual or calculated pixel count.
        count: u64,
        /// Configured maximum pixel count.
        max: u64,
    },

    /// Image dimensions or pixel count exceeds allowed threshold (legacy variant).
    #[error("Pixel limit exceeded: count={count}, max={max}")]
    PixelLimitExceeded {
        /// Actual or calculated pixel count.
        count: u64,
        /// Configured maximum pixel count.
        max: u64,
    },

    /// Estimated memory required for decoding/transformation exceeds limit.
    #[error("Memory limit exceeded: requested={requested}, max={max}")]
    LimitMemory {
        /// Estimated memory bytes required.
        requested: u64,
        /// Configured maximum memory bytes.
        max: u64,
    },

    /// Decoding the bitstream failed.
    #[error("Decoding failure: {0}")]
    DecodeError(String),

    /// Decoding failed (alias variant for error code mapping).
    #[error("Decoding failed: {0}")]
    DecodeFailed(String),

    /// Encoding to target format failed.
    #[error("Encoding failure: {0}")]
    EncodeError(String),

    /// Encoding failed (alias variant for error code mapping).
    #[error("Encoding failed: {0}")]
    EncodeFailed(String),

    /// The operation was aborted/cancelled by the caller.
    #[error("Operation aborted")]
    Aborted,

    /// The operation exceeded configured execution time.
    #[error("Operation timed out")]
    Timeout,

    /// Internal unexpected error.
    #[error("Internal error: {0}")]
    Internal(String),
}
