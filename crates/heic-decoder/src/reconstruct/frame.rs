//! Planar YUV frame buffer and format conversion to RGB/RGBA.

use valen_heic_core::{HeicResult, ImageDimensions, Limits, PixelFormat};
use valen_image_processing::PixelBuffer;

/// Planar YUV frame buffer (4:2:0 / 4:2:2).
#[derive(Debug, Clone)]
pub struct PlanarFrame {
    /// Luma plane samples.
    pub y: Vec<u16>,
    /// Luma stride in samples.
    pub y_stride: usize,
    /// Frame width in luma pixels.
    pub width: usize,
    /// Frame height in luma pixels.
    pub height: usize,
    /// Chroma Cb plane samples.
    pub cb: Vec<u16>,
    /// Chroma Cr plane samples.
    pub cr: Vec<u16>,
    /// Chroma stride in samples.
    pub c_stride: usize,
    /// Chroma width in pixels.
    pub chroma_width: usize,
    /// Chroma height in pixels.
    pub chroma_height: usize,
    /// Bit depth (8 or 10).
    pub bit_depth: u8,
}

impl PlanarFrame {
    /// Allocates an empty planar frame with limit checking.
    pub fn new(width: usize, height: usize, bit_depth: u8, limits: &Limits) -> HeicResult<Self> {
        let dims = ImageDimensions::new(width as u32, height as u32);
        limits.check_dimensions(dims)?;
        limits.check_pixel_count((width as u64).saturating_mul(height as u64))?;

        let y_stride = (width + 63) & !63; // Align to 64
        let y_alloc = y_stride * height;

        let chroma_width = width.div_ceil(2);
        let chroma_height = height.div_ceil(2);
        let c_stride = (chroma_width + 31) & !31;
        let c_alloc = c_stride * chroma_height;

        let total_bytes = (y_alloc + 2 * c_alloc) * if bit_depth > 8 { 2 } else { 1 };
        limits.check_memory_size(total_bytes as u64)?;

        let default_val = 1 << (bit_depth - 1); // 128 for 8-bit, 512 for 10-bit

        Ok(Self {
            y: vec![default_val; y_alloc],
            y_stride,
            width,
            height,
            cb: vec![default_val; c_alloc],
            cr: vec![default_val; c_alloc],
            c_stride,
            chroma_width,
            chroma_height,
            bit_depth,
        })
    }

    /// Converts planar YUV (4:2:0) to interleaved RGB/RGBA [`PixelBuffer`].
    pub fn to_pixel_buffer(&self, format: PixelFormat) -> PixelBuffer {
        let dims = ImageDimensions::new(self.width as u32, self.height as u32);
        let mut buffer = PixelBuffer::new(dims, format);

        let shift = self.bit_depth.saturating_sub(8);
        let bpp = format.bytes_per_pixel();

        for y in 0..self.height {
            let y_row = y * self.y_stride;
            let c_row = (y / 2) * self.c_stride;
            let out_row = y * buffer.stride;

            for x in 0..self.width {
                let y_val = (self.y[y_row + x] >> shift) as i32;
                let cb_val = (self.cb[c_row + (x / 2)] >> shift) as i32 - 128;
                let cr_val = (self.cr[c_row + (x / 2)] >> shift) as i32 - 128;

                // Standard ITU-R BT.601 / BT.709 integer YUV-to-RGB conversion
                let r = (y_val * 1024 + 1436 * cr_val + 512) >> 10;
                let g = (y_val * 1024 - 352 * cb_val - 731 * cr_val + 512) >> 10;
                let b = (y_val * 1024 + 1814 * cb_val + 512) >> 10;

                let r_u8 = r.clamp(0, 255) as u8;
                let g_u8 = g.clamp(0, 255) as u8;
                let b_u8 = b.clamp(0, 255) as u8;

                let px_idx = out_row + x * bpp;
                match format {
                    PixelFormat::Rgba8 => {
                        buffer.data[px_idx] = r_u8;
                        buffer.data[px_idx + 1] = g_u8;
                        buffer.data[px_idx + 2] = b_u8;
                        buffer.data[px_idx + 3] = 255;
                    }
                    PixelFormat::Rgb8 => {
                        buffer.data[px_idx] = r_u8;
                        buffer.data[px_idx + 1] = g_u8;
                        buffer.data[px_idx + 2] = b_u8;
                    }
                    PixelFormat::Bgra8 => {
                        buffer.data[px_idx] = b_u8;
                        buffer.data[px_idx + 1] = g_u8;
                        buffer.data[px_idx + 2] = r_u8;
                        buffer.data[px_idx + 3] = 255;
                    }
                    PixelFormat::Bgr8 => {
                        buffer.data[px_idx] = b_u8;
                        buffer.data[px_idx + 1] = g_u8;
                        buffer.data[px_idx + 2] = r_u8;
                    }
                    _ => {
                        // Default RGBA8 fallback
                        buffer.data[px_idx] = r_u8;
                        buffer.data[px_idx + 1] = g_u8;
                        buffer.data[px_idx + 2] = b_u8;
                        buffer.data[px_idx + 3] = 255;
                    }
                }
            }
        }

        buffer
    }
}
