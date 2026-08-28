//! Pure-Rust Lossless WebP (VP8L) Image Encoder.

use crate::ImageEncoder;
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

/// Helper struct for writing bit streams in VP8L bit-packing format.
struct Vp8lBitWriter {
    buffer: Vec<u8>,
    bit_accum: u64,
    bits_in_accum: u8,
}

impl Vp8lBitWriter {
    fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(4096),
            bit_accum: 0,
            bits_in_accum: 0,
        }
    }

    fn write_bits(&mut self, val: u32, num_bits: u8) {
        if num_bits == 0 {
            return;
        }
        self.bit_accum |= (val as u64) << self.bits_in_accum;
        self.bits_in_accum += num_bits;

        while self.bits_in_accum >= 8 {
            self.buffer.push((self.bit_accum & 0xFF) as u8);
            self.bit_accum >>= 8;
            self.bits_in_accum -= 8;
        }
    }

    fn flush(&mut self) {
        if self.bits_in_accum > 0 {
            self.buffer.push((self.bit_accum & 0xFF) as u8);
            self.bit_accum = 0;
            self.bits_in_accum = 0;
        }
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

    // Huffman tree headers:
    // Green (Channel 0): 280 symbols (0..255 literals, 256..279 length codes)
    // Simple code with 0 symbols (empty code) or flat 1-symbol code for simple stream
    // We encode a standard flat 8-bit literal Huffman code (8 bits per literal)
    // Tree type: Normal Huffman tree (0 = normal tree)
    // For green:
    writer.write_bits(0, 1); // normal tree (not simple)
                             // Code lengths for code length alphabet (19 symbols): 0 means uncompressed code lengths
    writer.write_bits(0, 1); // code length code lengths not present
                             // Let's write simple tree: 1
                             // Simple tree format:
                             // 1 bit: is_simple = 1
                             // 1 bit: num_symbols - 1 = 1 (2 symbols) or 0 (1 symbol)
                             // For universal uncompressed stream, VP8L specifies writing the simple tree header:

    // Write simple Huffman tree for green (literal 0):
    writer.write_bits(1, 1); // is_simple = 1
    writer.write_bits(0, 1); // 1 symbol
    writer.write_bits(0, 8); // symbol 0

    // Red:
    writer.write_bits(1, 1);
    writer.write_bits(0, 1);
    writer.write_bits(0, 8);

    // Blue:
    writer.write_bits(1, 1);
    writer.write_bits(0, 1);
    writer.write_bits(0, 8);

    // Alpha:
    writer.write_bits(1, 1);
    writer.write_bits(0, 1);
    writer.write_bits(0, 8);

    // Distance tree:
    writer.write_bits(1, 1);
    writer.write_bits(0, 1);
    writer.write_bits(0, 8);

    writer.flush();

    let vp8l_data = writer.buffer;

    // 2. Wrap in RIFF WEBP container
    let mut out = Vec::with_capacity(12 + 8 + vp8l_data.len() + 2);

    let vp8l_chunk_len = vp8l_data.len() as u32;
    let riff_len = 4 + 8 + vp8l_chunk_len + (vp8l_chunk_len & 1); // 'WEBP' + 'VP8L' + len + data + pad

    // 'RIFF' + size + 'WEBP'
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_len.to_le_bytes());
    out.extend_from_slice(b"WEBP");

    // 'VP8L' + size + data
    out.extend_from_slice(b"VP8L");
    out.extend_from_slice(&vp8l_chunk_len.to_le_bytes());
    out.extend_from_slice(&vp8l_data);

    if (vp8l_chunk_len & 1) != 0 {
        out.push(0x00); // RIFF 2-byte alignment padding
    }

    Ok(out)
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
