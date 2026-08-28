//! Error definitions and taxonomy for the HEIC/HEIF processing pipeline.

pub mod codes;
pub mod types;

pub use types::{HeicError, HeicResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let cases = [
            (HeicError::InvalidInput("test".into()), "INVALID_INPUT"),
            (
                HeicError::InvalidContainer("test".into()),
                "INVALID_CONTAINER",
            ),
            (HeicError::MalformedInput("test".into()), "MALFORMED_INPUT"),
            (
                HeicError::UnsupportedFormat("test".into()),
                "UNSUPPORTED_FORMAT",
            ),
            (
                HeicError::UnsupportedBrand("test".into()),
                "UNSUPPORTED_BRAND",
            ),
            (
                HeicError::UnsupportedCodec("test".into()),
                "UNSUPPORTED_CODEC",
            ),
            (
                HeicError::UnsupportedSequence("test".into()),
                "UNSUPPORTED_SEQUENCE",
            ),
            (
                HeicError::UnsupportedFeature("test".into()),
                "UNSUPPORTED_FEATURE",
            ),
            (HeicError::LimitExceeded("test".into()), "LIMIT_EXCEEDED"),
            (
                HeicError::LimitInputBytes {
                    actual: 200,
                    max: 100,
                },
                "LIMIT_INPUT_BYTES",
            ),
            (
                HeicError::LimitDimensions {
                    width: 4000,
                    height: 4000,
                    max_width: Some(2000),
                    max_height: Some(2000),
                },
                "LIMIT_DIMENSIONS",
            ),
            (
                HeicError::LimitPixels {
                    count: 1000,
                    max: 500,
                },
                "LIMIT_PIXELS",
            ),
            (
                HeicError::PixelLimitExceeded {
                    count: 1000,
                    max: 500,
                },
                "PIXEL_LIMIT_EXCEEDED",
            ),
            (
                HeicError::LimitMemory {
                    requested: 1000,
                    max: 500,
                },
                "LIMIT_MEMORY",
            ),
            (HeicError::DecodeError("test".into()), "DECODE_ERROR"),
            (HeicError::DecodeFailed("test".into()), "DECODE_FAILED"),
            (HeicError::EncodeError("test".into()), "ENCODE_ERROR"),
            (HeicError::EncodeFailed("test".into()), "ENCODE_FAILED"),
            (HeicError::Aborted, "OPERATION_ABORTED"),
            (HeicError::Timeout, "TIMEOUT"),
            (HeicError::Internal("test".into()), "INTERNAL_ERROR"),
        ];

        for (err, expected_code) in cases {
            assert_eq!(err.error_code(), expected_code);
        }
    }
}
