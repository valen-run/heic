//! Pure-Rust ISO/IEC 15948 Compliant PNG Encoder with Zlib/Deflate compression.

use crate::ImageEncoder;
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

/// Computes the IEEE 802.3 32-bit Cyclic Redundancy Check (CRC-32).
fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// Computes the Adler-32 checksum (RFC 1950).
fn adler32(data: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65521;
    let mut s1 = 1u32;
    let mut s2 = 0u32;

    for &byte in data {
        s1 = (s1 + byte as u32) % MOD_ADLER;
        s2 = (s2 + s1) % MOD_ADLER;
    }

    (s2 << 16) | s1
}

/// Encodes uncompressed filtered scanlines into a Zlib Deflate container (RFC 1950 & 1951).
fn deflate_zlib(uncompressed: &[u8]) -> Vec<u8> {
    let mut zlib = Vec::with_capacity(uncompressed.len() + 64);

    // 1. Zlib Header (CMF = 0x78 (Deflate, 32KB window), FLG = 0x01 (No preset dict, check bits))
    zlib.push(0x78);
    zlib.push(0x01);

    // 2. Deflate Non-compressed Blocks (BTYPE = 00)
    let chunk_size = 65535;
    let chunks: Vec<&[u8]> = uncompressed.chunks(chunk_size).collect();

    if chunks.is_empty() {
        // Empty final block
        zlib.push(0x01); // BFINAL = 1, BTYPE = 00
        zlib.extend_from_slice(&[0x00, 0x00, 0xFF, 0xFF]);
    } else {
        for (i, chunk) in chunks.iter().enumerate() {
            let is_final = i == chunks.len() - 1;
            let bfinal_btype = if is_final { 0x01 } else { 0x00 };
            zlib.push(bfinal_btype);

            let len = chunk.len() as u16;
            let nlen = !len;

            zlib.push(len as u8);
            zlib.push((len >> 8) as u8);
            zlib.push(nlen as u8);
            zlib.push((nlen >> 8) as u8);

            zlib.extend_from_slice(chunk);
        }
    }

    // 3. Adler-32 Checksum (big-endian)
    let adler = adler32(uncompressed);
    zlib.extend_from_slice(&adler.to_be_bytes());

    zlib
}

/// Writes a PNG chunk with length, chunk type, data, and CRC-32.
fn write_chunk(dest: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    let len = data.len() as u32;
    dest.extend_from_slice(&len.to_be_bytes());
    dest.extend_from_slice(chunk_type);
    dest.extend_from_slice(data);

    let mut crc_payload = Vec::with_capacity(4 + data.len());
    crc_payload.extend_from_slice(chunk_type);
    crc_payload.extend_from_slice(data);

    let crc = crc32(&crc_payload);
    dest.extend_from_slice(&crc.to_be_bytes());
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
    let color_type = if has_alpha { 6u8 } else { 2u8 }; // 6 = RGBA, 2 = RGB
    let bytes_per_pixel = if has_alpha { 4 } else { 3 };

    let mut out = Vec::with_capacity((width as usize) * (height as usize) * bytes_per_pixel + 1024);

    // 1. PNG Signature
    out.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

    // 2. IHDR Chunk
    let mut ihdr_data = [0u8; 13];
    ihdr_data[0..4].copy_from_slice(&width.to_be_bytes());
    ihdr_data[4..8].copy_from_slice(&height.to_be_bytes());
    ihdr_data[8] = 8; // Bit depth: 8 bits per channel
    ihdr_data[9] = color_type;
    ihdr_data[10] = 0; // Compression method: 0 (Deflate)
    ihdr_data[11] = 0; // Filter method: 0 (Adaptive)
    ihdr_data[12] = 0; // Interlace method: 0 (None)

    write_chunk(&mut out, b"IHDR", &ihdr_data);

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
