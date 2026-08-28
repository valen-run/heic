//! JPEG container marker emission (SOI, APP0, DQT, SOF0, DHT, SOS, EOI).

use super::huffman;
use super::quant::ZIGZAG;

/// Writes the standard JPEG headers before compressed bitstream payload.
pub fn write_headers(
    out: &mut Vec<u8>,
    width: usize,
    height: usize,
    q_luma: &[u8; 64],
    q_chroma: &[u8; 64],
) {
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
}
