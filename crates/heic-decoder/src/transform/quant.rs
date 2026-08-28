//! Inverse quantization scale factors and block dequantization (ITU-T H.265 Section 8.7.3).

/// Quantization scale factors for `qp % 6` (ITU-T H.265 Section 8.7.3).
pub static LEVEL_SCALE: [i32; 6] = [40, 45, 51, 57, 64, 72];

/// Dequantizes an `N x N` block of transform coefficients according to ITU-T H.265 Section 8.7.3.
pub fn dequantize_block(
    coeffs: &[i32],
    output: &mut [i32],
    size: usize,
    qp: i32,
    bit_depth: u8,
    transform_skip: bool,
) {
    let log2_size = match size {
        4 => 2,
        8 => 3,
        16 => 4,
        32 => 5,
        _ => 2,
    };

    let qp_rem = (qp % 6) as usize;
    let qp_per = qp / 6;
    let scale = LEVEL_SCALE[qp_rem];

    if transform_skip {
        let shift = (13 + 7 - bit_depth as i32) - qp_per;
        for i in 0..size * size {
            let coeff = coeffs[i];
            let scaled = (coeff * scale) << 4;
            if shift >= 0 {
                let add = 1 << (shift - 1);
                output[i] = (scaled + add) >> shift;
            } else {
                output[i] = scaled << (-shift);
            }
        }
        return;
    }

    let shift = (13 + log2_size - bit_depth as i32) - qp_per;
    for i in 0..size * size {
        let coeff = coeffs[i];
        let scaled = coeff * scale;
        if shift >= 0 {
            let add = 1 << (shift - 1);
            output[i] = (scaled + add) >> shift;
        } else {
            output[i] = scaled << (-shift);
        }
    }
}
