//! 2D image dimensions representation.

use std::fmt;

/// 2D image dimensions in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct ImageDimensions {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl ImageDimensions {
    /// Constructs new image dimensions.
    #[inline]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Returns `true` if either width or height is zero.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Computes total pixel count with overflow protection.
    ///
    /// Returns `None` if the multiplication overflows 64-bit integer space.
    #[inline]
    pub const fn checked_pixel_count(&self) -> Option<u64> {
        (self.width as u64).checked_mul(self.height as u64)
    }

    /// Computes total pixel count (`width * height`).
    ///
    /// Saturated at `u64::MAX` on arithmetic overflow.
    #[inline]
    pub const fn pixel_count(&self) -> u64 {
        match self.checked_pixel_count() {
            Some(count) => count,
            None => u64::MAX,
        }
    }

    /// Computes total required buffer byte size for a given number of bytes per pixel.
    ///
    /// Returns `None` if the calculation overflows `usize`.
    #[inline]
    pub fn checked_buffer_size(&self, bytes_per_pixel: usize) -> Option<usize> {
        let pixels = usize::try_from(self.checked_pixel_count()?).ok()?;
        pixels.checked_mul(bytes_per_pixel)
    }

    /// Returns the dimensions with width and height swapped (transposed).
    #[inline]
    pub const fn transposed(&self) -> Self {
        Self {
            width: self.height,
            height: self.width,
        }
    }
}

impl fmt::Display for ImageDimensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}
