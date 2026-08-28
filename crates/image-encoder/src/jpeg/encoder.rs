//! JPEG MCU traversal and block DCT entropy coding loop.

use super::color::YCbCrPlanes;
use super::dct::fdct_8x8;
use super::huffman::{encode_block, JpegBitWriter};

/// Bundle of compiled Huffman lookup tables for luminance and chrominance.
pub struct JpegHuffmanTables<'a> {
    /// DC luminance table.
    pub dc_luma: &'a [(u16, u8); 256],
    /// DC chrominance table.
    pub dc_chroma: &'a [(u16, u8); 256],
    /// AC luminance table.
    pub ac_luma: &'a [(u16, u8); 256],
    /// AC chrominance table.
    pub ac_chroma: &'a [(u16, u8); 256],
}

/// Compresses YCbCr planes into JPEG entropy-coded stream.
pub fn encode_mcu_stream(
    planes: &YCbCrPlanes,
    width: usize,
    height: usize,
    q_luma: &[u8; 64],
    q_chroma: &[u8; 64],
    huff_tables: &JpegHuffmanTables<'_>,
) -> Vec<u8> {
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

            for by in 0..8 {
                let py = (y0 + by).min(height - 1);
                for bx in 0..8 {
                    let px = (x0 + bx).min(width - 1);
                    block_y[by * 8 + bx] = planes.y[py * width + px];
                    block_cb[by * 8 + bx] = planes.cb[py * width + px];
                    block_cr[by * 8 + bx] = planes.cr[py * width + px];
                }
            }

            // Y Block
            fdct_8x8(&block_y, &mut dct_out);
            for i in 0..64 {
                quant_block[i] = (dct_out[i] / (q_luma[i] as f32)).round() as i32;
            }
            encode_block(
                &quant_block,
                &mut prev_dc_y,
                huff_tables.dc_luma,
                huff_tables.ac_luma,
                &mut writer,
            );

            // Cb Block
            fdct_8x8(&block_cb, &mut dct_out);
            for i in 0..64 {
                quant_block[i] = (dct_out[i] / (q_chroma[i] as f32)).round() as i32;
            }
            encode_block(
                &quant_block,
                &mut prev_dc_cb,
                huff_tables.dc_chroma,
                huff_tables.ac_chroma,
                &mut writer,
            );

            // Cr Block
            fdct_8x8(&block_cr, &mut dct_out);
            for i in 0..64 {
                quant_block[i] = (dct_out[i] / (q_chroma[i] as f32)).round() as i32;
            }
            encode_block(
                &quant_block,
                &mut prev_dc_cr,
                huff_tables.dc_chroma,
                huff_tables.ac_chroma,
                &mut writer,
            );
        }
    }

    writer.flush();
    writer.buffer
}
