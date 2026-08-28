//! Raw pixel buffer structures, stride representations, and blitting operations.

use valen_heic_core::{HeicError, HeicResult, ImageDimensions, Limits, PixelFormat};

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

    /// Creates a new pixel buffer validated against safety [`Limits`].
    pub fn new_with_limits(
        dimensions: ImageDimensions,
        format: PixelFormat,
        limits: &Limits,
    ) -> HeicResult<Self> {
        limits.check_dimensions(dimensions)?;
        limits.check_pixel_count(dimensions.pixel_count())?;

        let bpp = format.bytes_per_pixel();
        let stride = (dimensions.width as usize).saturating_mul(bpp);
        let total_bytes = stride.saturating_mul(dimensions.height as usize);
        limits.check_memory_size(total_bytes as u64)?;

        Ok(Self {
            dimensions,
            format,
            data: vec![0; total_bytes],
            stride,
        })
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

    /// Gets a slice of bytes representing the pixel at `(x, y)`.
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<&[u8]> {
        if x >= self.dimensions.width || y >= self.dimensions.height {
            return None;
        }
        let bpp = self.format.bytes_per_pixel();
        let start = (y as usize) * self.stride + (x as usize) * bpp;
        let end = start + bpp;
        if end <= self.data.len() {
            Some(&self.data[start..end])
        } else {
            None
        }
    }

    /// Sets the pixel at `(x, y)` from a slice of bytes.
    pub fn set_pixel(&mut self, x: u32, y: u32, pixel: &[u8]) -> HeicResult<()> {
        if x >= self.dimensions.width || y >= self.dimensions.height {
            return Err(HeicError::InvalidInput(
                "Pixel coordinates out of bounds".to_string(),
            ));
        }
        let bpp = self.format.bytes_per_pixel();
        if pixel.len() != bpp {
            return Err(HeicError::InvalidInput(format!(
                "Invalid pixel byte length: expected {}, got {}",
                bpp,
                pixel.len()
            )));
        }
        let start = (y as usize) * self.stride + (x as usize) * bpp;
        let end = start + bpp;
        if end <= self.data.len() {
            self.data[start..end].copy_from_slice(pixel);
            Ok(())
        } else {
            Err(HeicError::LimitExceeded(
                "Pixel index exceeds buffer capacity".to_string(),
            ))
        }
    }

    /// Copies a source sub-rectangle into this buffer at `(dst_x, dst_y)`.
    #[allow(clippy::too_many_arguments)]
    pub fn blit(
        &mut self,
        dst_x: u32,
        dst_y: u32,
        src: &PixelBuffer,
        src_x: u32,
        src_y: u32,
        width: u32,
        height: u32,
    ) -> HeicResult<()> {
        if self.format != src.format {
            return Err(HeicError::InvalidInput(
                "Cannot blit between pixel buffers of different formats".to_string(),
            ));
        }

        let bpp = self.format.bytes_per_pixel();
        let copy_w = width
            .min(src.dimensions.width.saturating_sub(src_x))
            .min(self.dimensions.width.saturating_sub(dst_x));
        let copy_h = height
            .min(src.dimensions.height.saturating_sub(src_y))
            .min(self.dimensions.height.saturating_sub(dst_y));

        let copy_bytes_per_row = (copy_w as usize) * bpp;

        for row in 0..copy_h {
            let s_row = ((src_y + row) as usize) * src.stride + ((src_x as usize) * bpp);
            let d_row = ((dst_y + row) as usize) * self.stride + ((dst_x as usize) * bpp);

            self.data[d_row..d_row + copy_bytes_per_row]
                .copy_from_slice(&src.data[s_row..s_row + copy_bytes_per_row]);
        }

        Ok(())
    }

    /// Crops this pixel buffer to a new dimension `(target_w, target_h)`.
    pub fn crop(&self, target_w: u32, target_h: u32, limits: &Limits) -> HeicResult<PixelBuffer> {
        let cropped_w = target_w.min(self.dimensions.width);
        let cropped_h = target_h.min(self.dimensions.height);
        let mut dest = PixelBuffer::new_with_limits(
            ImageDimensions::new(cropped_w, cropped_h),
            self.format,
            limits,
        )?;

        dest.blit(0, 0, self, 0, 0, cropped_w, cropped_h)?;
        Ok(dest)
    }

    /// Fills the entire buffer with a solid color byte pattern.
    pub fn fill(&mut self, color: &[u8]) -> HeicResult<()> {
        let bpp = self.format.bytes_per_pixel();
        if color.len() != bpp {
            return Err(HeicError::InvalidInput(format!(
                "Fill color size {} does not match format bpp {}",
                color.len(),
                bpp
            )));
        }

        for chunk in self.data.chunks_exact_mut(bpp) {
            chunk.copy_from_slice(color);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_buffer_creation_and_pixel_access() {
        let mut buf = PixelBuffer::new(ImageDimensions::new(10, 10), PixelFormat::Rgb8);
        assert_eq!(buf.dimensions.width, 10);
        assert_eq!(buf.dimensions.height, 10);
        assert_eq!(buf.stride, 30);
        assert_eq!(buf.data.len(), 300);

        let red = [255, 0, 0];
        buf.set_pixel(2, 3, &red).expect("Set pixel should succeed");
        assert_eq!(buf.get_pixel(2, 3), Some(&red[..]));
        assert_eq!(buf.get_pixel(0, 0), Some(&[0, 0, 0][..]));
        assert_eq!(buf.get_pixel(10, 10), None);
    }

    #[test]
    fn test_pixel_buffer_blit() {
        let mut dest = PixelBuffer::new(ImageDimensions::new(20, 20), PixelFormat::Rgba8);
        let mut src = PixelBuffer::new(ImageDimensions::new(10, 10), PixelFormat::Rgba8);
        src.fill(&[10, 20, 30, 255]).expect("Fill should succeed");

        dest.blit(5, 5, &src, 0, 0, 10, 10)
            .expect("Blit should succeed");

        assert_eq!(dest.get_pixel(0, 0), Some(&[0, 0, 0, 0][..]));
        assert_eq!(dest.get_pixel(5, 5), Some(&[10, 20, 30, 255][..]));
        assert_eq!(dest.get_pixel(14, 14), Some(&[10, 20, 30, 255][..]));
        assert_eq!(dest.get_pixel(15, 15), Some(&[0, 0, 0, 0][..]));
    }
}
