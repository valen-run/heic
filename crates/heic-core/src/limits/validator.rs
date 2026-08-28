//! Resource and dimension limit validation logic.

use super::math::checked_mul;
use super::Limits;
use crate::error::{HeicError, HeicResult};
use crate::types::{ImageDimensions, PixelFormat};

impl Limits {
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

    /// Estimates the memory required to store uncompressed pixels for the given dimensions and format.
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
