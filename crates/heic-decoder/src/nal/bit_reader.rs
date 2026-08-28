//! Safe bit-level reader over an RBSP byte slice with Exp-Golomb decoding.

use valen_heic_core::{HeicError, HeicResult};

/// Safe bit-level reader over an RBSP byte slice.
#[derive(Debug, Clone)]
pub struct BitReader<'a> {
    bytes: &'a [u8],
    /// Current byte offset in input.
    pub byte_offset: usize,
    /// Current bit offset within the byte (0..8).
    pub bit_offset: u8,
}

impl<'a> BitReader<'a> {
    /// Creates a new [`BitReader`] over an RBSP buffer.
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            byte_offset: 0,
            bit_offset: 0,
        }
    }

    /// Returns `true` if no more bits are available.
    pub fn is_empty(&self) -> bool {
        self.byte_offset >= self.bytes.len()
    }

    /// Number of remaining bits in the reader.
    pub fn remaining_bits(&self) -> usize {
        if self.byte_offset >= self.bytes.len() {
            0
        } else {
            (self.bytes.len() - self.byte_offset) * 8 - (self.bit_offset as usize)
        }
    }

    /// Reads a single bit (0 or 1).
    pub fn read_bit(&mut self) -> HeicResult<u8> {
        if self.byte_offset >= self.bytes.len() {
            return Err(HeicError::MalformedInput(
                "Unexpected end of RBSP bitstream".into(),
            ));
        }

        let bit = (self.bytes[self.byte_offset] >> (7 - self.bit_offset)) & 1;
        self.bit_offset += 1;
        if self.bit_offset == 8 {
            self.bit_offset = 0;
            self.byte_offset += 1;
        }
        Ok(bit)
    }

    /// Reads `n` bits (up to 32) as a `u32`.
    pub fn read_bits(&mut self, n: u8) -> HeicResult<u32> {
        if n == 0 {
            return Ok(0);
        }
        if n > 32 {
            return Err(HeicError::MalformedInput(
                "Cannot read more than 32 bits at once".into(),
            ));
        }

        let mut val = 0u32;
        for _ in 0..n {
            val = (val << 1) | (self.read_bit()? as u32);
        }
        Ok(val)
    }

    /// Reads an unsigned Exp-Golomb code `ue(v)`.
    pub fn read_ue(&mut self) -> HeicResult<u32> {
        let mut leading_zeros = 0u8;
        while self.read_bit()? == 0 {
            leading_zeros += 1;
            if leading_zeros > 31 {
                return Err(HeicError::MalformedInput(
                    "Exp-Golomb code exceeds 32-bit integer capacity".into(),
                ));
            }
        }

        if leading_zeros == 0 {
            return Ok(0);
        }

        let info = self.read_bits(leading_zeros)?;
        let val = (1u32 << leading_zeros).wrapping_sub(1).wrapping_add(info);
        Ok(val)
    }

    /// Reads a signed Exp-Golomb code `se(v)`.
    pub fn read_se(&mut self) -> HeicResult<i32> {
        let code_num = self.read_ue()?;
        if code_num == 0 {
            Ok(0)
        } else if (code_num & 1) != 0 {
            Ok(code_num.div_ceil(2) as i32)
        } else {
            Ok(-((code_num / 2) as i32))
        }
    }

    /// Aligns bit pointer to next byte boundary.
    pub fn byte_align(&mut self) {
        if self.bit_offset != 0 {
            self.bit_offset = 0;
            self.byte_offset += 1;
        }
    }

    /// Checks if there is more RBSP data before the stop bit.
    pub fn has_more_rbsp_data(&self) -> bool {
        if self.byte_offset >= self.bytes.len() {
            return false;
        }
        if self.byte_offset == self.bytes.len() - 1 {
            let last_byte = self.bytes[self.byte_offset];
            let remaining_in_byte = 8 - self.bit_offset;
            // A stop bit is '1' followed by all '0's
            let mask = (1 << remaining_in_byte) - 1;
            let val = last_byte & mask;
            let stop_bit_pattern = 1 << (remaining_in_byte.saturating_sub(1));
            return val != stop_bit_pattern;
        }
        true
    }
}
