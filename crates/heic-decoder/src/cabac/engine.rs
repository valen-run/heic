//! Binary Arithmetic Decoding Engine (Section 9.3.3).

use crate::cabac::context::ContextModel;
use crate::cabac::tables::{RANGE_TAB_LPS, TRANS_IDX_LPS, TRANS_IDX_MPS};
use valen_heic_core::{HeicError, HeicResult};

/// Binary Arithmetic Decoding Engine.
#[derive(Debug)]
pub struct CabacEngine<'a> {
    bytes: &'a [u8],
    byte_idx: usize,
    ivl_curr_range: u32,
    ivl_offset: u32,
    bits_needed: i32,
}

impl<'a> CabacEngine<'a> {
    /// Initializes CABAC decoding engine from byte stream at start of slice data (Section 9.3.2.5).
    pub fn init(bytes: &'a [u8], start_byte_offset: usize) -> HeicResult<Self> {
        let mut engine = Self {
            bytes,
            byte_idx: start_byte_offset,
            ivl_curr_range: 510,
            ivl_offset: 0,
            bits_needed: 0,
        };

        // Read initial 9 bits into ivl_offset
        let b0 = engine.read_byte()?;
        let b1 = engine.read_byte()?;
        engine.ivl_offset = ((b0 as u32) << 1) | ((b1 as u32) >> 7);
        engine.bits_needed = 7;

        Ok(engine)
    }

    fn read_byte(&mut self) -> HeicResult<u8> {
        if self.byte_idx < self.bytes.len() {
            let b = self.bytes[self.byte_idx];
            self.byte_idx += 1;
            Ok(b)
        } else {
            // Emulate trailing zeros if bitstream ended
            Ok(0)
        }
    }

    /// Decodes a regular context-coded binary decision (Section 9.3.3.2).
    pub fn decode_bin(&mut self, ctx: &mut ContextModel) -> HeicResult<u8> {
        let q_range_idx = ((self.ivl_curr_range >> 6) & 3) as usize;
        let ivl_lps_range = RANGE_TAB_LPS[ctx.state as usize][q_range_idx] as u32;
        let mut ivl_curr_range = self.ivl_curr_range - ivl_lps_range;

        let bin_val;
        if self.ivl_offset < ivl_curr_range {
            bin_val = ctx.val_mps;
            ctx.state = TRANS_IDX_MPS[ctx.state as usize];
        } else {
            bin_val = 1 - ctx.val_mps;
            self.ivl_offset -= ivl_curr_range;
            ivl_curr_range = ivl_lps_range;
            if ctx.state == 0 {
                ctx.val_mps = 1 - ctx.val_mps;
            }
            ctx.state = TRANS_IDX_LPS[ctx.state as usize];
        }

        // Renormalization
        while ivl_curr_range < 256 {
            ivl_curr_range <<= 1;
            self.ivl_offset <<= 1;
            self.bits_needed -= 1;
            if self.bits_needed < 0 {
                let next_byte = self.read_byte()? as u32;
                self.ivl_offset |= (next_byte >> 7) & 1;
                self.bits_needed = 7;
            } else if self.byte_idx > 0 && self.byte_idx <= self.bytes.len() + 1 {
                let cur_byte = if self.byte_idx <= self.bytes.len() {
                    self.bytes[self.byte_idx - 1]
                } else {
                    0
                } as u32;
                self.ivl_offset |= (cur_byte >> self.bits_needed) & 1;
            }
        }

        self.ivl_curr_range = ivl_curr_range;
        Ok(bin_val)
    }

    /// Decodes an equiprobable bypass bin (Section 9.3.3.4).
    pub fn decode_bypass_bin(&mut self) -> HeicResult<u8> {
        self.ivl_offset <<= 1;
        self.bits_needed -= 1;
        if self.bits_needed < 0 {
            let next_byte = self.read_byte()? as u32;
            self.ivl_offset |= (next_byte >> 7) & 1;
            self.bits_needed = 7;
        } else if self.byte_idx > 0 && self.byte_idx <= self.bytes.len() + 1 {
            let cur_byte = if self.byte_idx <= self.bytes.len() {
                self.bytes[self.byte_idx - 1]
            } else {
                0
            } as u32;
            self.ivl_offset |= (cur_byte >> self.bits_needed) & 1;
        }

        let bin_val = if self.ivl_offset >= self.ivl_curr_range {
            self.ivl_offset -= self.ivl_curr_range;
            1
        } else {
            0
        };

        Ok(bin_val)
    }

    /// Decodes `n` consecutive bypass bins as a `u32`.
    pub fn decode_bins_bypass(&mut self, n: u8) -> HeicResult<u32> {
        let mut val = 0u32;
        for _ in 0..n {
            val = (val << 1) | (self.decode_bypass_bin()? as u32);
        }
        Ok(val)
    }

    /// Decodes a terminating bin (end of slice or end of CTU).
    pub fn decode_terminate_bin(&mut self) -> HeicResult<u8> {
        self.ivl_curr_range -= 2;
        if self.ivl_offset >= self.ivl_curr_range {
            Ok(1)
        } else {
            while self.ivl_curr_range < 256 {
                self.ivl_curr_range <<= 1;
                self.ivl_offset <<= 1;
                self.bits_needed -= 1;
                if self.bits_needed < 0 {
                    let next_byte = self.read_byte()? as u32;
                    self.ivl_offset |= (next_byte >> 7) & 1;
                    self.bits_needed = 7;
                } else if self.byte_idx > 0 && self.byte_idx <= self.bytes.len() + 1 {
                    let cur_byte = if self.byte_idx <= self.bytes.len() {
                        self.bytes[self.byte_idx - 1]
                    } else {
                        0
                    } as u32;
                    self.ivl_offset |= (cur_byte >> self.bits_needed) & 1;
                }
            }
            Ok(0)
        }
    }

    /// Decodes `coeff_abs_level_remaining` syntax element (Section 9.3.3.7).
    pub fn decode_coeff_abs_level_remaining(&mut self, c_rice_param: u32) -> HeicResult<u32> {
        let mut prefix = 0u32;
        while self.decode_bypass_bin()? != 0 {
            prefix += 1;
            if prefix > 31 {
                return Err(HeicError::MalformedInput(
                    "Exceeded max escape prefix".into(),
                ));
            }
        }

        if prefix < 3 {
            let suffix = self.decode_bins_bypass(c_rice_param as u8)?;
            Ok((prefix << c_rice_param) + suffix)
        } else {
            let prefix_minus3 = prefix - 3;
            let ext_bits = prefix_minus3 + c_rice_param;
            let suffix = self.decode_bins_bypass(ext_bits as u8)?;
            let code = (((1 << prefix_minus3) + 3 - 1) << c_rice_param) + suffix;
            Ok(code)
        }
    }
}
