//! Pure-Rust Baseline Sequential DCT JPEG Encoder (ISO/IEC 10918-1 / ITU-T T.81).

pub mod color;
pub mod dct;
pub mod encoder;
pub mod huffman;
pub mod quant;
pub mod writer;

use crate::ImageEncoder;
use color::rgb_to_ycbcr_planes;
use encoder::encode_mcu_stream;
use valen_heic_core::{HeicError, HeicResult};
use valen_image_processing::PixelBuffer;
use writer::write_headers;

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

    write_headers(&mut out, width, height, &q_luma, &q_chroma);

    let planes = rgb_to_ycbcr_planes(buffer, width, height);

    let huff_tables = encoder::JpegHuffmanTables {
        dc_luma: &dc_luma_lut,
        dc_chroma: &dc_chroma_lut,
        ac_luma: &ac_luma_lut,
        ac_chroma: &ac_chroma_lut,
    };

    let entropy_data = encode_mcu_stream(&planes, width, height, &q_luma, &q_chroma, &huff_tables);

    out.extend_from_slice(&entropy_data);

    // EOI Marker
    out.extend_from_slice(&[0xFF, 0xD9]);

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use valen_heic_core::{ImageDimensions, PixelFormat};

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
