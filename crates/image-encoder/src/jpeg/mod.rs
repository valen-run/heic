//! Pure-Rust Baseline Sequential DCT JPEG Encoder (ISO/IEC 10918-1 / ITU-T T.81).

pub mod dct;
pub mod huffman;
pub mod quant;

use crate::ImageEncoder;
use valen_heic_core::{HeicError, HeicResult, PixelFormat};
use valen_image_processing::PixelBuffer;

pub use dct::fdct_8x8;
pub use huffman::{build_huffman_lut, encode_block, JpegBitWriter};
pub use quant::{scale_quant_table, STD_CHROMA_QTABLE, STD_LUMA_QTABLE, ZIGZAG};

/// Encoding options for JPEG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JpegOptions {
    /// JPEG quality from 1 to 100 (default 85).
    pub quality: u8,
}

impl Default for JpegOptions {
    fn default() -> Self {
        Self { quality: 85 }
    }
}

/// Baseline Sequential DCT JPEG encoder.
#[derive(Debug, Default, Clone, Copy)]
pub struct JpegEncoder {
    /// Encoding options.
    pub options: JpegOptions,
}

impl JpegEncoder {
    /// Creates a new JPEG encoder with default options.
    pub const fn new() -> Self {
        Self {
            options: JpegOptions { quality: 85 },
        }
    }

    /// Creates a new JPEG encoder with specific quality.
    pub const fn with_quality(quality: u8) -> Self {
        Self {
            options: JpegOptions {
                quality: if quality == 0 {
                    1
                } else if quality > 100 {
                    100
                } else {
                    quality
                },
            },
        }
    }
}

impl ImageEncoder for JpegEncoder {
    type Options = JpegOptions;

    fn encode(&self, buffer: &PixelBuffer, options: &Self::Options) -> HeicResult<Vec<u8>> {
        encode_jpeg(buffer, options.quality)
    }
}

/// Encodes an uncompressed [`PixelBuffer`] into standard JPEG bytes.
pub fn encode_jpeg(buffer: &PixelBuffer, quality: u8) -> HeicResult<Vec<u8>> {
    let width = buffer.dimensions.width as usize;
    let height = buffer.dimensions.height as usize;

    if width == 0 || height == 0 {
        return Err(HeicError::InvalidInput(
            "Image dimensions cannot be zero".into(),
        ));
    }

    let q_luma = scale_quant_table(&STD_LUMA_QTABLE, quality);
    let q_chroma = scale_quant_table(&STD_CHROMA_QTABLE, quality);

    let dc_luma_lut = build_huffman_lut(&huffman::DC_LUMA_BITS, &huffman::DC_LUMA_HUFFVAL);
    let dc_chroma_lut = build_huffman_lut(&huffman::DC_CHROMA_BITS, &huffman::DC_CHROMA_HUFFVAL);
    let ac_luma_lut = build_huffman_lut(&huffman::AC_LUMA_BITS, &huffman::AC_LUMA_HUFFVAL);
    let ac_chroma_lut = build_huffman_lut(&huffman::AC_CHROMA_BITS, &huffman::AC_CHROMA_HUFFVAL);

    let mut out = Vec::with_capacity(width * height / 2);

    // 1. SOI Marker
    out.extend_from_slice(&[0xFF, 0xD8]);

    // 2. APP0 JFIF Marker
    out.extend_from_slice(&[
        0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00,
        0x01, 0x00, 0x00,
    ]);

    // 3. DQT (Quantization Tables)
    out.extend_from_slice(&[0xFF, 0xDB, 0x00, 0x84]);
    out.push(0x00); // Luma Table 0
    for &idx in &ZIGZAG {
        out.push(q_luma[idx]);
    }
    out.push(0x01); // Chroma Table 1
    for &idx in &ZIGZAG {
        out.push(q_chroma[idx]);
    }

    // 4. SOF0 (Baseline DCT)
    out.extend_from_slice(&[
        0xFF,
        0xC0,
        0x00,
        0x11,
        0x08, // 8-bit sample precision
        (height >> 8) as u8,
        height as u8,
        (width >> 8) as u8,
        width as u8,
        3, // 3 components (Y, Cb, Cr)
        1,
        0x11,
        0, // Y: 1x1 sampling, table 0
        2,
        0x11,
        1, // Cb: 1x1 sampling, table 1
        3,
        0x11,
        1, // Cr: 1x1 sampling, table 1
    ]);

    // 5. DHT (Huffman Tables)
    // DC Luma
    out.extend_from_slice(&[0xFF, 0xC4, 0x00, 0x1F, 0x00]);
    out.extend_from_slice(&huffman::DC_LUMA_BITS);
    out.extend_from_slice(&huffman::DC_LUMA_HUFFVAL);

    // AC Luma
    let ac_luma_len = 3 + 16 + huffman::AC_LUMA_HUFFVAL.len();
    out.extend_from_slice(&[
        0xFF,
        0xC4,
        (ac_luma_len >> 8) as u8,
        ac_luma_len as u8,
        0x10,
    ]);
    out.extend_from_slice(&huffman::AC_LUMA_BITS);
    out.extend_from_slice(&huffman::AC_LUMA_HUFFVAL);

    // DC Chroma
    out.extend_from_slice(&[0xFF, 0xC4, 0x00, 0x1F, 0x01]);
    out.extend_from_slice(&huffman::DC_CHROMA_BITS);
    out.extend_from_slice(&huffman::DC_CHROMA_HUFFVAL);

    // AC Chroma
    let ac_chroma_len = 3 + 16 + huffman::AC_CHROMA_HUFFVAL.len();
    out.extend_from_slice(&[
        0xFF,
        0xC4,
        (ac_chroma_len >> 8) as u8,
        ac_chroma_len as u8,
        0x11,
    ]);
    out.extend_from_slice(&huffman::AC_CHROMA_BITS);
    out.extend_from_slice(&huffman::AC_CHROMA_HUFFVAL);

    // 6. SOS (Start of Scan)
    out.extend_from_slice(&[
        0xFF, 0xDA, 0x00, 0x0C, 3, // 3 components
        1, 0x00, // Y uses DC 0, AC 0
        2, 0x11, // Cb uses DC 1, AC 1
        3, 0x11, // Cr uses DC 1, AC 1
        0, 63, 0, // Spectral selection & point transform
    ]);

    // Convert pixel buffer to planar Y, Cb, Cr floats
    let mut y_plane = vec![0.0f32; width * height];
    let mut cb_plane = vec![0.0f32; width * height];
    let mut cr_plane = vec![0.0f32; width * height];

    let bpp = buffer.format.bytes_per_pixel();

    for y in 0..height {
        let row_start = y * buffer.stride;
        for x in 0..width {
            let idx = row_start + x * bpp;
            let (r, g, b) = match buffer.format {
                PixelFormat::Rgb8 => (
                    buffer.data[idx] as f32,
                    buffer.data[idx + 1] as f32,
                    buffer.data[idx + 2] as f32,
                ),
                PixelFormat::Rgba8 => {
                    let a = buffer.data[idx + 3] as f32 / 255.0;
                    (
                        buffer.data[idx] as f32 * a + 255.0 * (1.0 - a),
                        buffer.data[idx + 1] as f32 * a + 255.0 * (1.0 - a),
                        buffer.data[idx + 2] as f32 * a + 255.0 * (1.0 - a),
                    )
                }
                PixelFormat::Bgr8 => (
                    buffer.data[idx + 2] as f32,
                    buffer.data[idx + 1] as f32,
                    buffer.data[idx] as f32,
                ),
                PixelFormat::Bgra8 => {
                    let a = buffer.data[idx + 3] as f32 / 255.0;
                    (
                        buffer.data[idx + 2] as f32 * a + 255.0 * (1.0 - a),
                        buffer.data[idx + 1] as f32 * a + 255.0 * (1.0 - a),
                        buffer.data[idx] as f32 * a + 255.0 * (1.0 - a),
                    )
                }
                _ => (
                    buffer.data[idx] as f32,
                    buffer.data[idx] as f32,
                    buffer.data[idx] as f32,
                ),
            };

            let y_val = 0.299 * r + 0.587 * g + 0.114 * b - 128.0;
            let cb_val = -0.168736 * r - 0.331264 * g + 0.5 * b;
            let cr_val = 0.5 * r - 0.418688 * g - 0.081312 * b;

            y_plane[y * width + x] = y_val;
            cb_plane[y * width + x] = cb_val;
            cr_plane[y * width + x] = cr_val;
        }
    }

    // 7. Entropy-code MCU blocks (8x8 for Y, Cb, Cr)
    let mut writer = JpegBitWriter::new();
    let mut prev_dc_y = 0i32;
    let mut prev_dc_cb = 0i32;
    let mut prev_dc_cr = 0i32;

    let mcu_cols = width.div_ceil(8);
    let mcu_rows = height.div_ceil(8);

    let mut block_y = [0f32; 64];
    let mut block_cb = [0f32; 64];
    let mut block_cr = [0f32; 64];
    let mut dct_out = [0f32; 64];
    let mut quant_block = [0i32; 64];

    for mcu_r in 0..mcu_rows {
        for mcu_c in 0..mcu_cols {
            let x0 = mcu_c * 8;
            let y0 = mcu_r * 8;

            // Extract 8x8 block with edge clamping
            for by in 0..8 {
                let py = (y0 + by).min(height - 1);
                for bx in 0..8 {
                    let px = (x0 + bx).min(width - 1);
                    block_y[by * 8 + bx] = y_plane[py * width + px];
                    block_cb[by * 8 + bx] = cb_plane[py * width + px];
                    block_cr[by * 8 + bx] = cr_plane[py * width + px];
                }
            }

            // Encode Y Block
            fdct_8x8(&block_y, &mut dct_out);
            for i in 0..64 {
                quant_block[i] = (dct_out[i] / (q_luma[i] as f32)).round() as i32;
            }
            encode_block(
                &quant_block,
                &mut prev_dc_y,
                &dc_luma_lut,
                &ac_luma_lut,
                &mut writer,
            );

            // Encode Cb Block
            fdct_8x8(&block_cb, &mut dct_out);
            for i in 0..64 {
                quant_block[i] = (dct_out[i] / (q_chroma[i] as f32)).round() as i32;
            }
            encode_block(
                &quant_block,
                &mut prev_dc_cb,
                &dc_chroma_lut,
                &ac_chroma_lut,
                &mut writer,
            );

            // Encode Cr Block
            fdct_8x8(&block_cr, &mut dct_out);
            for i in 0..64 {
                quant_block[i] = (dct_out[i] / (q_chroma[i] as f32)).round() as i32;
            }
            encode_block(
                &quant_block,
                &mut prev_dc_cr,
                &dc_chroma_lut,
                &ac_chroma_lut,
                &mut writer,
            );
        }
    }

    writer.flush();
    out.extend_from_slice(&writer.buffer);

    // 8. EOI Marker
    out.extend_from_slice(&[0xFF, 0xD9]);

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use valen_heic_core::ImageDimensions;

    #[test]
    fn test_encode_jpeg_headers() {
        let mut buf = PixelBuffer::new(ImageDimensions::new(8, 8), PixelFormat::Rgb8);
        buf.fill(&[128, 64, 32]).unwrap();

        let jpeg_bytes = encode_jpeg(&buf, 85).expect("JPEG encode should succeed");
        assert!(jpeg_bytes.starts_with(&[0xFF, 0xD8])); // SOI
        assert!(jpeg_bytes.ends_with(&[0xFF, 0xD9])); // EOI
    }

    #[test]
    fn test_jpeg_quality_scaling() {
        let mut buf = PixelBuffer::new(ImageDimensions::new(32, 32), PixelFormat::Rgb8);
        for y in 0..32 {
            for x in 0..32 {
                buf.set_pixel(x, y, &[(x * 8) as u8, (y * 8) as u8, ((x + y) * 4) as u8])
                    .unwrap();
            }
        }

        let low_q = encode_jpeg(&buf, 10).unwrap();
        let high_q = encode_jpeg(&buf, 95).unwrap();

        assert!(low_q.len() < high_q.len());
    }
}
