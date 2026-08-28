//! Bitstream writer for Lossless WebP (VP8L) bit-packing.

/// Helper struct for writing bit streams in VP8L bit-packing format.
#[derive(Debug, Default)]
pub struct Vp8lBitWriter {
    /// Accumulated encoded bytes.
    pub buffer: Vec<u8>,
    bit_accum: u64,
    bits_in_accum: u8,
}

impl Vp8lBitWriter {
    /// Creates a new bitstream writer with initial buffer capacity.
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(4096),
            bit_accum: 0,
            bits_in_accum: 0,
        }
    }

    /// Writes `num_bits` of `val` into the bitstream in LSB-first order.
    pub fn write_bits(&mut self, val: u32, num_bits: u8) {
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

    /// Flushes any pending bits into buffer byte with zero padding.
    pub fn flush(&mut self) {
        if self.bits_in_accum > 0 {
            self.buffer.push((self.bit_accum & 0xFF) as u8);
            self.bit_accum = 0;
            self.bits_in_accum = 0;
        }
    }
}
