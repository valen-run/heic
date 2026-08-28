//! Pure-Rust ISO/IEC 15948 Compliant PNG Encoder with Zlib/Deflate compression.

pub mod chunks;
pub mod deflate;

use crate::ImageEncoder;
use chunks::{write_chunk, write_ihdr};
use deflate::deflate_zlib;
use valen_heic_core::{HeicError, HeicResult, PixelFormat};
use valen_image_processing::PixelBuffer;

/// Encoding options for PNG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PngOptions {
    /// Compression level from 0 to 9 (default 6).
    pub compression_level: u8,
}

/// PNG image encoder.
#[derive(Debug, Default, Clone, Copy)]
pub struct PngEncoder;

impl PngEncoder {
    /// Creates a new PNG encoder instance.
    pub const fn new() -> Self {
        Self
    }
}

impl ImageEncoder for PngEncoder {
    type Options = PngOptions;

    fn encode(&self, buffer: &PixelBuffer, options: &Self::Options) -> HeicResult<Vec<u8>> {
        encode_png(buffer, options)
    }
}

/// Encodes an uncompressed [`PixelBuffer`] into standard PNG binary bytes.
pub fn encode_png(buffer: &PixelBuffer, _options: &PngOptions) -> HeicResult<Vec<u8>> {
    let width = buffer.dimensions.width;
    let height = buffer.dimensions.height;

    if width == 0 || height == 0 {
        return Err(HeicError::InvalidInput(
            "Image dimensions cannot be zero".into(),
        ));
    }

    let has_alpha = buffer.format.has_alpha();
    let color_type = if has_alpha { 6u8 } else { 2u8 };
    let bytes_per_pixel = if has_alpha { 4 } else { 3 };

    let mut out = Vec::with_capacity((width as usize) * (height as usize) * bytes_per_pixel + 1024);

    // 1. PNG Signature
    out.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

    // 2. IHDR Chunk
    write_ihdr(&mut out, width, height, color_type);

    // 3. Prepare filtered scanlines (Filter 0 = None)
    let row_len = 1 + (width as usize) * bytes_per_pixel;
    let mut raw_scanlines = Vec::with_capacity((height as usize) * row_len);
    let src_bpp = buffer.format.bytes_per_pixel();

    for y in 0..height {
        raw_scanlines.push(0); // Filter type 0 (None)
        let row_start = (y as usize) * buffer.stride;

        for x in 0..width {
            let src_idx = row_start + (x as usize) * src_bpp;

            let (r, g, b, a) = match buffer.format {
                PixelFormat::Rgb8 => (
                    buffer.data[src_idx],
                    buffer.data[src_idx + 1],
                    buffer.data[src_idx + 2],
                    255u8,
                ),
                PixelFormat::Rgba8 => (
                    buffer.data[src_idx],
                    buffer.data[src_idx + 1],
                    buffer.data[src_idx + 2],
                    buffer.data[src_idx + 3],
                ),
                PixelFormat::Bgr8 => (
                    buffer.data[src_idx + 2],
                    buffer.data[src_idx + 1],
                    buffer.data[src_idx],
                    255u8,
                ),
                PixelFormat::Bgra8 => (
                    buffer.data[src_idx + 2],
                    buffer.data[src_idx + 1],
                    buffer.data[src_idx],
                    buffer.data[src_idx + 3],
                ),
                _ => (
                    buffer.data[src_idx],
                    buffer.data[src_idx],
                    buffer.data[src_idx],
                    255u8,
                ),
            };

            raw_scanlines.push(r);
            raw_scanlines.push(g);
            raw_scanlines.push(b);
            if has_alpha {
                raw_scanlines.push(a);
            }
        }
    }

    // 4. IDAT Chunk (Zlib compressed scanlines)
    let compressed_idat = deflate_zlib(&raw_scanlines);
    write_chunk(&mut out, b"IDAT", &compressed_idat);

    // 5. IEND Chunk
    write_chunk(&mut out, b"IEND", &[]);

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use valen_heic_core::ImageDimensions;

    #[test]
    fn test_encode_png_rgb() {
        let mut buf = PixelBuffer::new(ImageDimensions::new(4, 4), PixelFormat::Rgb8);
        buf.fill(&[255, 128, 0]).unwrap();

        let png_bytes =
            encode_png(&buf, &PngOptions::default()).expect("PNG encode should succeed");
        assert!(png_bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]));
        assert!(png_bytes.windows(4).any(|w| w == b"IHDR"));
        assert!(png_bytes.windows(4).any(|w| w == b"IDAT"));
        assert!(png_bytes.windows(4).any(|w| w == b"IEND"));
    }

    #[test]
    fn test_encode_png_rgba() {
        let mut buf = PixelBuffer::new(ImageDimensions::new(2, 2), PixelFormat::Rgba8);
        buf.fill(&[0, 255, 128, 64]).unwrap();

        let png_bytes =
            encode_png(&buf, &PngOptions::default()).expect("PNG encode should succeed");
        assert!(png_bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]));
    }
}
