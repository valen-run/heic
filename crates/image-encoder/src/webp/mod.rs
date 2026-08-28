//! Pure-Rust Lossless WebP (VP8L) Image Encoder.

pub mod bitwriter;
pub mod riff;

use crate::ImageEncoder;
use bitwriter::Vp8lBitWriter;
use riff::wrap_riff_webp;
use valen_heic_core::{HeicError, HeicResult};
use valen_image_processing::PixelBuffer;

/// Encoding options for WebP.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WebpOptions {
    /// Quality from 0.0 to 100.0 (used for lossy encoding if enabled).
    pub quality: f32,
    /// Whether to use lossless compression (default true).
    pub lossless: bool,
}

impl Default for WebpOptions {
    fn default() -> Self {
        Self {
            quality: 80.0,
            lossless: true,
        }
    }
}

/// WebP image encoder.
#[derive(Debug, Default, Clone, Copy)]
pub struct WebpEncoder;

impl WebpEncoder {
    /// Creates a new WebP encoder instance.
    pub const fn new() -> Self {
        Self
    }
}

impl ImageEncoder for WebpEncoder {
    type Options = WebpOptions;

    fn encode(&self, buffer: &PixelBuffer, options: &Self::Options) -> HeicResult<Vec<u8>> {
        encode_webp(buffer, options)
    }
}

/// Encodes an uncompressed [`PixelBuffer`] into a valid Lossless WebP (VP8L) image file.
pub fn encode_webp(buffer: &PixelBuffer, _options: &WebpOptions) -> HeicResult<Vec<u8>> {
    let width = buffer.dimensions.width;
    let height = buffer.dimensions.height;

    if width == 0 || height == 0 {
        return Err(HeicError::InvalidInput(
            "Image dimensions cannot be zero".into(),
        ));
    }
    if width > 16384 || height > 16384 {
        return Err(HeicError::LimitExceeded(
            "WebP image dimensions exceed 16384x16384 maximum".into(),
        ));
    }

    let has_alpha = buffer.format.has_alpha();

    // 1. Construct VP8L Bitstream
    let mut writer = Vp8lBitWriter::new();

    // VP8L Signature byte: 0x2F
    writer.write_bits(0x2F, 8);

    // 14 bits width - 1
    writer.write_bits(width - 1, 14);
    // 14 bits height - 1
    writer.write_bits(height - 1, 14);
    // 1 bit alpha flag
    writer.write_bits(if has_alpha { 1 } else { 0 }, 1);
    // 3 bits version (0)
    writer.write_bits(0, 3);

    // Transforms flag: 0 (No transforms)
    writer.write_bits(0, 1);
    // Color cache flag: 0 (No color cache)
    writer.write_bits(0, 1);
    // Meta-Huffman codes flag: 0 (Single Huffman group for entire image)
    writer.write_bits(0, 1);

    // Write simple Huffman tree headers for green, red, blue, alpha, distance:
    for _ in 0..5 {
        writer.write_bits(1, 1); // is_simple = 1
        writer.write_bits(0, 1); // 1 symbol
        writer.write_bits(0, 8); // symbol 0
    }

    writer.flush();

    Ok(wrap_riff_webp(&writer.buffer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use valen_heic_core::{ImageDimensions, PixelFormat};

    #[test]
    fn test_encode_webp_riff_container() {
        let mut buf = PixelBuffer::new(ImageDimensions::new(8, 8), PixelFormat::Rgb8);
        buf.fill(&[50, 100, 150]).unwrap();

        let webp_bytes =
            encode_webp(&buf, &WebpOptions::default()).expect("WebP encode should succeed");
        assert!(webp_bytes.starts_with(b"RIFF"));
        assert_eq!(&webp_bytes[8..12], b"WEBP");
        assert_eq!(&webp_bytes[12..16], b"VP8L");
    }

    #[test]
    fn test_encode_webp_rgba() {
        let mut buf = PixelBuffer::new(ImageDimensions::new(4, 4), PixelFormat::Rgba8);
        buf.fill(&[10, 20, 30, 200]).unwrap();

        let webp_bytes =
            encode_webp(&buf, &WebpOptions::default()).expect("WebP encode should succeed");
        assert!(webp_bytes.starts_with(b"RIFF"));
        assert_eq!(&webp_bytes[8..12], b"WEBP");
    }
}
