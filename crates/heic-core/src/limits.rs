//! Configurable safety and resource limits for parsing and decoding operations.

use crate::error::{HeicError, HeicResult};
use crate::types::{ImageDimensions, PixelFormat};

/// Default maximum input file size for browser environments (256 MiB).
pub const DEFAULT_MAX_FILE_SIZE: u64 = 256 * 1024 * 1024;

/// Default maximum dimension (width or height) in pixels (16,384 px).
pub const DEFAULT_MAX_DIMENSION: u32 = 16_384;

/// Default maximum total pixel count (67,108,864 pixels / 64 Mpx).
pub const DEFAULT_MAX_PIXEL_COUNT: u64 = 64 * 1024 * 1024;

/// Default maximum decoded buffer memory (512 MiB).
pub const DEFAULT_MAX_MEMORY_BYTES: u64 = 512 * 1024 * 1024;

/// Default maximum container items/boxes to parse (defensive anti-DoS limit).
pub const DEFAULT_MAX_ITEM_COUNT: usize = 10_000;

/// Default maximum grid tile count.
pub const DEFAULT_MAX_TILE_COUNT: usize = 1_024;

/// Performs checked multiplication of two `u64` integers, returning a [`HeicError::LimitExceeded`] on overflow.
#[inline]
pub fn checked_mul(a: u64, b: u64) -> HeicResult<u64> {
    a.checked_mul(b).ok_or_else(|| {
        HeicError::LimitExceeded(format!("Integer overflow in arithmetic: {a} * {b}"))
    })
}

/// Performs checked multiplication of two `usize` integers, returning a [`HeicError::LimitExceeded`] on overflow.
#[inline]
pub fn checked_mul_usize(a: usize, b: usize) -> HeicResult<usize> {
    a.checked_mul(b).ok_or_else(|| {
        HeicError::LimitExceeded(format!(
            "Integer overflow in buffer size calculation: {a} * {b}"
        ))
    })
}

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
    /// Maximum number of items in ISO-BMFF meta containers.
    pub max_item_count: Option<usize>,
    /// Maximum number of grid tiles allowed.
    pub max_tile_count: Option<usize>,
}

impl Limits {
    /// Creates a new `Limits` instance with no constraints (unbounded).
    pub fn none() -> Self {
        Self::default()
    }

    /// Creates a new `Limits` instance configured with safe browser defaults.
    pub fn default_browser() -> Self {
        Self {
            max_file_size: Some(DEFAULT_MAX_FILE_SIZE),
            max_width: Some(DEFAULT_MAX_DIMENSION),
            max_height: Some(DEFAULT_MAX_DIMENSION),
            max_pixel_count: Some(DEFAULT_MAX_PIXEL_COUNT),
            max_memory_bytes: Some(DEFAULT_MAX_MEMORY_BYTES),
            max_item_count: Some(DEFAULT_MAX_ITEM_COUNT),
            max_tile_count: Some(DEFAULT_MAX_TILE_COUNT),
        }
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

    /// Set the maximum item count.
    pub fn with_max_item_count(mut self, count: usize) -> Self {
        self.max_item_count = Some(count);
        self
    }

    /// Set the maximum grid tile count.
    pub fn with_max_tile_count(mut self, count: usize) -> Self {
        self.max_tile_count = Some(count);
        self
    }

    /// Validates an input buffer size against `max_file_size`.
    pub fn check_file_size(&self, actual_size: u64) -> HeicResult<()> {
        if let Some(max) = self.max_file_size {
            if actual_size > max {
                return Err(HeicError::LimitInputBytes {
                    actual: actual_size,
                    max,
                });
            }
        }
        Ok(())
    }

    /// Validates image dimensions and total pixel count before memory allocation.
    pub fn check_dimensions(&self, dimensions: ImageDimensions) -> HeicResult<()> {
        if let Some(max_w) = self.max_width {
            if dimensions.width > max_w {
                return Err(HeicError::LimitDimensions {
                    width: dimensions.width,
                    height: dimensions.height,
                    max_width: self.max_width,
                    max_height: self.max_height,
                });
            }
        }

        if let Some(max_h) = self.max_height {
            if dimensions.height > max_h {
                return Err(HeicError::LimitDimensions {
                    width: dimensions.width,
                    height: dimensions.height,
                    max_width: self.max_width,
                    max_height: self.max_height,
                });
            }
        }

        let pixel_count = dimensions
            .checked_pixel_count()
            .ok_or(HeicError::LimitDimensions {
                width: dimensions.width,
                height: dimensions.height,
                max_width: self.max_width,
                max_height: self.max_height,
            })?;

        self.check_pixel_count(pixel_count)?;

        Ok(())
    }

    /// Validates a pixel count against `max_pixel_count`.
    pub fn check_pixel_count(&self, pixel_count: u64) -> HeicResult<()> {
        if let Some(max_pixels) = self.max_pixel_count {
            if pixel_count > max_pixels {
                return Err(HeicError::LimitPixels {
                    count: pixel_count,
                    max: max_pixels,
                });
            }
        }
        Ok(())
    }

    /// Estimates the memory required to store uncompressed pixels for the given dimensions and format,
    /// checking for integer overflows and verifying against `max_memory_bytes`.
    pub fn estimate_memory(
        &self,
        dimensions: ImageDimensions,
        format: PixelFormat,
    ) -> HeicResult<u64> {
        let pixels = dimensions
            .checked_pixel_count()
            .ok_or(HeicError::LimitDimensions {
                width: dimensions.width,
                height: dimensions.height,
                max_width: self.max_width,
                max_height: self.max_height,
            })?;

        let bpp = format.bytes_per_pixel() as u64;
        let estimated_bytes = checked_mul(pixels, bpp)?;

        self.check_memory_size(estimated_bytes)?;

        Ok(estimated_bytes)
    }

    /// Validates expected buffer memory against `max_memory_bytes`.
    pub fn check_memory_size(&self, estimated_bytes: u64) -> HeicResult<()> {
        if let Some(max) = self.max_memory_bytes {
            if estimated_bytes > max {
                return Err(HeicError::LimitMemory {
                    requested: estimated_bytes,
                    max,
                });
            }
        }
        Ok(())
    }

    /// Validates container item count against `max_item_count`.
    pub fn check_item_count(&self, count: usize) -> HeicResult<()> {
        if let Some(max) = self.max_item_count {
            if count > max {
                return Err(HeicError::LimitExceeded(format!(
                    "Item count {count} exceeds limit of {max}"
                )));
            }
        }
        Ok(())
    }

    /// Validates grid tile count against `max_tile_count`.
    pub fn check_tile_count(&self, count: usize) -> HeicResult<()> {
        if let Some(max) = self.max_tile_count {
            if count > max {
                return Err(HeicError::LimitExceeded(format!(
                    "Tile count {count} exceeds limit of {max}"
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

    #[test]
    fn test_overflow_protection_u32_max() {
        let limits = Limits::none();
        let max_dim = ImageDimensions::new(u32::MAX, u32::MAX);

        // Checked pixel count should not panic, but return None or huge count
        assert!(max_dim.checked_pixel_count().is_some()); // (2^32 - 1)^2 fits in u64

        // But estimate_memory with 4 or 8 bytes per pixel will overflow u64
        let res = limits.estimate_memory(max_dim, PixelFormat::Rgba10);
        assert!(res.is_err());

        // Check dimensions with default browser limits rejects u32::MAX
        let browser_limits = Limits::default_browser();
        assert!(browser_limits.check_dimensions(max_dim).is_err());
    }

    #[test]
    fn test_estimate_memory_valid() {
        let limits = Limits::none().with_max_memory_bytes(10_000_000);
        let dim = ImageDimensions::new(1000, 1000); // 1,000,000 pixels

        // RGB8: 3,000,000 bytes <= 10,000,000
        let mem_rgb = limits.estimate_memory(dim, PixelFormat::Rgb8).unwrap();
        assert_eq!(mem_rgb, 3_000_000);

        // RGBA10: 8,000,000 bytes <= 10,000,000
        let mem_rgba10 = limits.estimate_memory(dim, PixelFormat::Rgba10).unwrap();
        assert_eq!(mem_rgba10, 8_000_000);

        // 2000x2000 RGBA8: 16,000,000 bytes > 10,000,000
        let dim_large = ImageDimensions::new(2000, 2000);
        assert!(limits
            .estimate_memory(dim_large, PixelFormat::Rgba8)
            .is_err());
    }

    #[test]
    fn test_item_and_tile_limits() {
        let limits = Limits::none()
            .with_max_item_count(100)
            .with_max_tile_count(16);

        assert!(limits.check_item_count(50).is_ok());
        assert!(limits.check_item_count(101).is_err());

        assert!(limits.check_tile_count(16).is_ok());
        assert!(limits.check_tile_count(17).is_err());
    }

    #[test]
    fn test_checked_arithmetic() {
        assert_eq!(checked_mul(10, 20).unwrap(), 200);
        assert!(checked_mul(u64::MAX, 2).is_err());

        assert_eq!(checked_mul_usize(10, 20).unwrap(), 200);
        assert!(checked_mul_usize(usize::MAX, 2).is_err());
    }
}
