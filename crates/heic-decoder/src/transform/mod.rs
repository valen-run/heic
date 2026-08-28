//! Inverse quantization, 4x4 DST-VII, and 4x4/8x8/16x16/32x32 DCT-II transforms (ITU-T H.265 Section 8.7).

pub mod dct;
pub mod dst;
pub mod quant;

pub use dct::{
    dct32_coeff, inverse_dct16, inverse_dct32, inverse_dct4, inverse_dct8, DCT16, DCT4, DCT8,
};
pub use dst::{inverse_dst4, DST4};
pub use quant::{dequantize_block, LEVEL_SCALE};

/// Applies 2D inverse integer transform to dequantized residual coefficients (Section 8.7.4).
pub fn inverse_transform(
    input: &[i32],
    output: &mut [i32],
    size: usize,
    is_luma: bool,
    mode: u8,
    bit_depth: u8,
    transform_skip: bool,
) {
    if transform_skip {
        output[..size * size].copy_from_slice(&input[..size * size]);
        return;
    }

    // 4x4 intra luma uses DST-VII (Section 8.7.4.1)
    if size == 4 && is_luma && mode < 35 {
        inverse_dst4(input, output, bit_depth);
        return;
    }

    match size {
        4 => inverse_dct4(input, output, bit_depth),
        8 => inverse_dct8(input, output, bit_depth),
        16 => inverse_dct16(input, output, bit_depth),
        32 => inverse_dct32(input, output, bit_depth),
        _ => output[..size * size].copy_from_slice(&input[..size * size]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dst4_identity() {
        let mut input = [0i32; 16];
        input[0] = 512;
        let mut output = [0i32; 16];
        inverse_dst4(&input, &mut output, 8);
        assert_ne!(output[0], 0);
    }

    #[test]
    fn test_dct4_identity() {
        let mut input = [0i32; 16];
        input[0] = 64;
        let mut output = [0i32; 16];
        inverse_dct4(&input, &mut output, 8);
        for &val in &output {
            assert_eq!(val, 1);
        }
    }
}
