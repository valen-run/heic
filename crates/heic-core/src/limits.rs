//! Configurable safety and resource limits for parsing and decoding operations.

use crate::error::{HeicError, HeicResult};
use crate::types::ImageDimensions;

/// Configurable resource limits to guard against decompression bombs and excessive memory consumption.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Limits {
    /// Maximum allowed input file size in bytes.
    pub max_file_size: Option<u64>,
    /// Maximum allowed image width in pixels.
    pub max_width: Option<u32>,
    /// Maximum allowed image height in pixels.
    pub max_height: Option<u32>,
    /// Maximum allowed total pixel count (`width * height`).
    pub max_pixel_count: Option<u64>,
    /// Maximum memory limit in bytes for intermediate or decoded pixel buffers.
    pub max_memory_bytes: Option<u64>,
}

impl Limits {
    /// Creates a new `Limits` with no constraints.
    pub fn none() -> Self {
        Self::default()
    }

    /// Set the maximum input file size in bytes.
    pub fn with_max_file_size(mut self, bytes: u64) -> Self {
        self.max_file_size = Some(bytes);
        self
    }

    /// Set the maximum width in pixels.
    pub fn with_max_width(mut self, width: u32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Set the maximum height in pixels.
    pub fn with_max_height(mut self, height: u32) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Set the maximum total pixel count.
    pub fn with_max_pixel_count(mut self, pixel_count: u64) -> Self {
        self.max_pixel_count = Some(pixel_count);
        self
    }

    /// Set the maximum decoded buffer memory in bytes.
    pub fn with_max_memory_bytes(mut self, bytes: u64) -> Self {
        self.max_memory_bytes = Some(bytes);
        self
    }

    /// Validates an input buffer size against `max_file_size`.
    pub fn check_file_size(&self, actual_size: u64) -> HeicResult<()> {
        if let Some(max) = self.max_file_size {
            if actual_size > max {
                return Err(HeicError::LimitExceeded(format!(
                    "File size {actual_size} bytes exceeds limit of {max} bytes"
                )));
            }
        }
        Ok(())
    }

    /// Validates image dimensions and total pixel count before memory allocation.
    pub fn check_dimensions(&self, dimensions: ImageDimensions) -> HeicResult<()> {
        if let Some(max_w) = self.max_width {
            if dimensions.width > max_w {
                return Err(HeicError::LimitExceeded(format!(
                    "Image width {} exceeds limit of {}",
                    dimensions.width, max_w
                )));
            }
        }

        if let Some(max_h) = self.max_height {
            if dimensions.height > max_h {
                return Err(HeicError::LimitExceeded(format!(
                    "Image height {} exceeds limit of {}",
                    dimensions.height, max_h
                )));
            }
        }

        let pixel_count = dimensions.pixel_count();
        if let Some(max_pixels) = self.max_pixel_count {
            if pixel_count > max_pixels {
                return Err(HeicError::PixelLimitExceeded {
                    count: pixel_count,
                    max: max_pixels,
                });
            }
        }

        Ok(())
    }

    /// Validates expected buffer memory against `max_memory_bytes`.
    pub fn check_memory_size(&self, estimated_bytes: u64) -> HeicResult<()> {
        if let Some(max) = self.max_memory_bytes {
            if estimated_bytes > max {
                return Err(HeicError::LimitExceeded(format!(
                    "Estimated memory {estimated_bytes} bytes exceeds limit of {max} bytes"
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_size_limit() {
        let limits = Limits::none().with_max_file_size(100);
        assert!(limits.check_file_size(50).is_ok());
        assert!(limits.check_file_size(100).is_ok());
        assert!(limits.check_file_size(101).is_err());
    }

    #[test]
    fn test_dimension_limits() {
        let limits = Limits::none()
            .with_max_width(1920)
            .with_max_height(1080)
            .with_max_pixel_count(2_000_000);

        let valid = ImageDimensions {
            width: 1920,
            height: 1080,
        };
        // 1920 * 1080 = 2,073,600 > 2,000,000
        assert!(limits.check_dimensions(valid).is_err());

        let smaller = ImageDimensions {
            width: 1280,
            height: 720,
        };
        assert!(limits.check_dimensions(smaller).is_ok());
    }
}
