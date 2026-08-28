//! Configurable safety and resource limits for parsing and decoding operations.

pub mod defaults;
pub mod math;
pub mod validator;

pub use defaults::{
    DEFAULT_MAX_DIMENSION, DEFAULT_MAX_FILE_SIZE, DEFAULT_MAX_ITEM_COUNT, DEFAULT_MAX_MEMORY_BYTES,
    DEFAULT_MAX_PIXEL_COUNT, DEFAULT_MAX_TILE_COUNT,
};
pub use math::{checked_mul, checked_mul_usize};

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ImageDimensions, PixelFormat};

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
        assert!(max_dim.checked_pixel_count().is_some());
        let res = limits.estimate_memory(max_dim, PixelFormat::Rgba10);
        assert!(res.is_err());
        let browser_limits = Limits::default_browser();
        assert!(browser_limits.check_dimensions(max_dim).is_err());
    }

    #[test]
    fn test_estimate_memory_valid() {
        let limits = Limits::none().with_max_memory_bytes(10_000_000);
        let dim = ImageDimensions::new(1000, 1000);
        let mem_rgb = limits.estimate_memory(dim, PixelFormat::Rgb8).unwrap();
        assert_eq!(mem_rgb, 3_000_000);
        let mem_rgba10 = limits.estimate_memory(dim, PixelFormat::Rgba10).unwrap();
        assert_eq!(mem_rgba10, 8_000_000);
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
