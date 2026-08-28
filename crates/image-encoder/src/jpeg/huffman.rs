//! Huffman lookup tables, bitstream writing, and MCU block encoding.

use crate::jpeg::quant::ZIGZAG;

/// Standard DC Luminance Huffman code lengths and values.
pub static DC_LUMA_BITS: [u8; 16] = [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];
/// Standard DC Luminance Huffman values.
pub static DC_LUMA_HUFFVAL: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

/// Standard DC Chrominance Huffman code lengths and values.
pub static DC_CHROMA_BITS: [u8; 16] = [0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0];
/// Standard DC Chrominance Huffman values.
pub static DC_CHROMA_HUFFVAL: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

/// Standard AC Luminance Huffman table.
pub static AC_LUMA_BITS: [u8; 16] = [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7D];
/// Standard AC Luminance Huffman values.
pub static AC_LUMA_HUFFVAL: [u8; 162] = [
    0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07,
    0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08, 0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0,
    0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0A, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x25, 0x26, 0x27, 0x28,
    0x29, 0x2A, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
    0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69,
    0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
    0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7,
    0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5,
    0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1, 0xE2,
    0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8,
    0xF9, 0xFA,
];

/// Standard AC Chrominance Huffman table.
pub static AC_CHROMA_BITS: [u8; 16] = [0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 0x77];
/// Standard AC Chrominance Huffman values.
pub static AC_CHROMA_HUFFVAL: [u8; 162] = [
    0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61, 0x71,
    0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42, 0x91, 0xA1, 0xB1, 0xC1, 0x09, 0x23, 0x33, 0x52, 0xF0,
    0x15, 0x62, 0x72, 0xD1, 0x0A, 0x16, 0x24, 0x34, 0xE1, 0x25, 0xF1, 0x17, 0x18, 0x19, 0x1A, 0x26,
    0x27, 0x28, 0x29, 0x2A, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
    0x49, 0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
    0x69, 0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
    0x88, 0x89, 0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5,
    0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3,
    0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA,
    0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8,
    0xF9, 0xFA,
];

/// Helper struct for writing bit streams with JPEG 0xFF byte stuffing.
pub struct JpegBitWriter {
    /// Encoded bytes.
    pub buffer: Vec<u8>,
    bit_accum: u32,
    bits_in_accum: u8,
}

impl Default for JpegBitWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl JpegBitWriter {
    /// Creates a new empty bit writer.
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(4096),
            bit_accum: 0,
            bits_in_accum: 0,
        }
    }

    /// Writes `len` bits from `code` into the bitstream.
    pub fn write_bits(&mut self, code: u16, len: u8) {
        if len == 0 {
            return;
        }
        self.bit_accum = (self.bit_accum << len) | (code as u32 & ((1 << len) - 1));
        self.bits_in_accum += len;

        while self.bits_in_accum >= 8 {
            let byte = ((self.bit_accum >> (self.bits_in_accum - 8)) & 0xFF) as u8;
            self.buffer.push(byte);
            if byte == 0xFF {
                self.buffer.push(0x00); // Byte stuffing
            }
            self.bits_in_accum -= 8;
        }
    }

    /// Flushes any remaining unwritten bits aligned to the next byte boundary.
    pub fn flush(&mut self) {
        if self.bits_in_accum > 0 {
            let byte = ((self.bit_accum << (8 - self.bits_in_accum)) & 0xFF) as u8;
            self.buffer.push(byte);
            if byte == 0xFF {
                self.buffer.push(0x00);
            }
            self.bit_accum = 0;
            self.bits_in_accum = 0;
        }
    }
}

/// Generates Huffman lookup table for fast encoding (returns `(code, length)` indexed by symbol).
pub fn build_huffman_lut(bits: &[u8; 16], huffval: &[u8]) -> [(u16, u8); 256] {
    let mut lut = [(0u16, 0u8); 256];
    let mut code = 0u16;
    let mut k = 0;

    for (len_idx, &count) in bits.iter().enumerate() {
        let bit_len = (len_idx + 1) as u8;
        for _ in 0..count {
            if k < huffval.len() {
                let val = huffval[k] as usize;
                lut[val] = (code, bit_len);
                code += 1;
                k += 1;
            }
        }
        code <<= 1;
    }
    lut
}

/// Encodes coefficient magnitude category and value bits according to Table F.1.
pub fn encode_coeff_bits(val: i32) -> (u8, u16) {
    if val == 0 {
        return (0, 0);
    }
    let abs_v = val.unsigned_abs();
    let num_bits = 32 - abs_v.leading_zeros() as u8;
    let code = if val < 0 {
        (val - 1 + (1 << num_bits)) as u16
    } else {
        val as u16
    };
    (num_bits, code)
}

/// Encodes an 8x8 quantized block with Huffman DC and AC codes.
pub fn encode_block(
    quant: &[i32; 64],
    prev_dc: &mut i32,
    dc_lut: &[(u16, u8); 256],
    ac_lut: &[(u16, u8); 256],
    writer: &mut JpegBitWriter,
) {
    // 1. DC Coefficient
    let dc_val = quant[0];
    let diff = dc_val - *prev_dc;
    *prev_dc = dc_val;

    let (dc_len, dc_bits) = encode_coeff_bits(diff);
    let (huff_code, huff_len) = dc_lut[dc_len as usize];
    writer.write_bits(huff_code, huff_len);
    writer.write_bits(dc_bits, dc_len);

    // 2. AC Coefficients (Zigzag scan 1..64)
    let mut run = 0u8;
    for k in 1..64 {
        let val = quant[ZIGZAG[k]];
        if val == 0 {
            run += 1;
        } else {
            while run >= 16 {
                let (code, len) = ac_lut[0xF0]; // ZRL (16 zeros)
                writer.write_bits(code, len);
                run -= 16;
            }
            let (coeff_len, coeff_bits) = encode_coeff_bits(val);
            let symbol = (run << 4) | coeff_len;
            let (code, len) = ac_lut[symbol as usize];
            writer.write_bits(code, len);
            writer.write_bits(coeff_bits, coeff_len);
            run = 0;
        }
    }

    if run > 0 {
        let (code, len) = ac_lut[0x00]; // EOB (End of Block)
        writer.write_bits(code, len);
    }
}
