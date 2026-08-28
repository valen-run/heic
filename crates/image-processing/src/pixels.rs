//! Raw pixel buffer structures and stride representations.

use valen_heic_core::{ImageDimensions, PixelFormat};

/// Represents an in-memory decoded raw pixel buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelBuffer {
    /// Dimensions of the image buffer.
    pub dimensions: ImageDimensions,
    /// Pixel format of the buffer.
    pub format: PixelFormat,
    /// Raw bytes containing the interleaved pixel data.
    pub data: Vec<u8>,
    /// Row stride in bytes (bytes per line).
    pub stride: usize,
}

impl PixelBuffer {
    /// Creates a new empty pixel buffer with the given dimensions and format.
    pub fn new(dimensions: ImageDimensions, format: PixelFormat) -> Self {
        let bpp = format.bytes_per_pixel();
        let stride = (dimensions.width as usize) * bpp;
        let total_bytes = stride * (dimensions.height as usize);
        Self {
            dimensions,
            format,
            data: vec![0; total_bytes],
            stride,
        }
    }

    /// Creates a pixel buffer from existing raw bytes after verifying length.
    pub fn from_raw(
        dimensions: ImageDimensions,
        format: PixelFormat,
        data: Vec<u8>,
    ) -> Result<Self, String> {
        let bpp = format.bytes_per_pixel();
        let expected_stride = (dimensions.width as usize) * bpp;
        let expected_len = expected_stride * (dimensions.height as usize);

        if data.len() != expected_len {
            return Err(format!(
                "Buffer size mismatch: expected {} bytes, got {}",
                expected_len,
                data.len()
            ));
        }

        Ok(Self {
            dimensions,
            format,
            data,
            stride: expected_stride,
        })
    }
}
