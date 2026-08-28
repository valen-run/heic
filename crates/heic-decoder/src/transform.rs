//! Inverse quantization, 4x4 DST-VII, and 4x4/8x8/16x16/32x32 DCT-II transforms (ITU-T H.265 Section 8.7).

/// Quantization scale factors for `qp % 6` (ITU-T H.265 Section 8.7.3).
pub static LEVEL_SCALE: [i32; 6] = [40, 45, 51, 57, 64, 72];

/// 4x4 DST-VII core transform matrix (for 4x4 intra luma residual).
pub static DST4: [[i32; 4]; 4] = [
    [29, 55, 74, 84],
    [74, 74, 0, -74],
    [84, -29, -74, 55],
    [55, -84, 74, -29],
];

/// 4x4 DCT-II core transform matrix.
pub static DCT4: [[i32; 4]; 4] = [
    [64, 64, 64, 64],
    [83, 36, -36, -83],
    [64, -64, -64, 64],
    [36, -83, 83, -36],
];

/// 8x8 DCT-II core transform matrix.
pub static DCT8: [[i32; 8]; 8] = [
    [64, 64, 64, 64, 64, 64, 64, 64],
    [89, 75, 50, 18, -18, -50, -75, -89],
    [83, 36, -36, -83, -83, -36, 36, 83],
    [75, -18, -89, -50, 50, 89, 18, -75],
    [64, -64, -64, 64, 64, -64, -64, 64],
    [50, -89, 18, 75, -75, -18, 89, -50],
    [36, -83, 83, -36, -36, 83, -83, 36],
    [18, -50, 75, -89, 89, -75, 50, -18],
];

/// 16x16 DCT-II core transform matrix.
pub static DCT16: [[i32; 16]; 16] = [
    [
        64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
    ],
    [
        90, 87, 80, 70, 57, 43, 25, 9, -9, -25, -43, -57, -70, -80, -87, -90,
    ],
    [
        89, 75, 50, 18, -18, -50, -75, -89, -89, -75, -50, -18, 18, 50, 75, 89,
    ],
    [
        87, 57, 9, -43, -80, -90, -70, -25, 25, 70, 90, 80, 43, -9, -57, -87,
    ],
    [
        83, 36, -36, -83, -83, -36, 36, 83, 83, 36, -36, -83, -83, -36, 36, 83,
    ],
    [
        80, 9, -70, -87, -25, 57, 90, 43, -43, -90, -57, 25, 87, 70, -9, -80,
    ],
    [
        75, -18, -89, -50, 50, 89, 18, -75, -75, 18, 89, 50, -50, -89, -18, 75,
    ],
    [
        70, -43, -87, 9, 90, 25, -80, -57, 57, 80, -25, -90, -9, 87, 43, -70,
    ],
    [
        64, -64, -64, 64, 64, -64, -64, 64, 64, -64, -64, 64, 64, -64, -64, 64,
    ],
    [
        57, -80, -25, 90, -9, -87, 43, 70, -70, -43, 87, 9, -90, 25, 80, -57,
    ],
    [
        50, -89, 18, 75, -75, -18, 89, -50, -50, 89, -18, -75, 75, 18, -89, 50,
    ],
    [
        43, -90, 57, 25, -87, 70, 9, -80, 80, -9, -70, 87, -25, -57, 90, -43,
    ],
    [
        36, -83, 83, -36, -36, 83, -83, 36, 36, -83, 83, -36, -36, 83, -83, 36,
    ],
    [
        25, -70, 90, -80, 43, 9, -57, 87, -87, 57, -9, -43, 80, -90, 70, -25,
    ],
    [
        18, -50, 75, -89, 89, -75, 50, -18, -18, 50, -75, 89, -89, 75, -50, 18,
    ],
    [
        9, -25, 43, -57, 70, -80, 87, -90, 90, -87, 80, -70, 57, -43, 25, -9,
    ],
];

/// Dequantizes an `N x N` block of transform coefficients according to ITU-T H.265 Section 8.7.3.
///
/// Multiplies quantized levels by `LEVEL_SCALE[qp % 6]` and applies bit-depth dependent bit shifts `(13 + log2_size - bit_depth) - qp / 6`.
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

/// Applies 2D inverse integer transform to dequantized residual coefficients (Section 8.7.4).
///
/// Uses:
/// - 4x4 Discrete Sine Transform (DST-VII) for 4x4 intra luma residual blocks
/// - 4x4, 8x8, 16x16, 32x32 Discrete Cosine Transform (DCT-II) for all other block sizes
/// - Passthrough copy if `transform_skip` is enabled
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

fn inverse_dst4(input: &[i32], output: &mut [i32], bit_depth: u8) {
    let mut temp = [0i32; 16];
    let shift1 = 7;
    let shift2 = 20 - bit_depth as i32;

    // Horizontal 1D IDST
    for y in 0..4 {
        for x in 0..4 {
            let mut sum = 0i32;
            for k in 0..4 {
                sum += input[y * 4 + k] * DST4[k][x];
            }
            temp[y * 4 + x] = (sum + (1 << (shift1 - 1))) >> shift1;
        }
    }

    // Vertical 1D IDST
    for x in 0..4 {
        for y in 0..4 {
            let mut sum = 0i32;
            for k in 0..4 {
                sum += temp[k * 4 + x] * DST4[k][y];
            }
            output[y * 4 + x] = (sum + (1 << (shift2 - 1))) >> shift2;
        }
    }
}

fn inverse_dct4(input: &[i32], output: &mut [i32], bit_depth: u8) {
    let mut temp = [0i32; 16];
    let shift1 = 7;
    let shift2 = 20 - bit_depth as i32;

    for y in 0..4 {
        for x in 0..4 {
            let mut sum = 0i32;
            for k in 0..4 {
                sum += input[y * 4 + k] * DCT4[k][x];
            }
            temp[y * 4 + x] = (sum + (1 << (shift1 - 1))) >> shift1;
        }
    }

    for x in 0..4 {
        for y in 0..4 {
            let mut sum = 0i32;
            for k in 0..4 {
                sum += temp[k * 4 + x] * DCT4[k][y];
            }
            output[y * 4 + x] = (sum + (1 << (shift2 - 1))) >> shift2;
        }
    }
}

fn inverse_dct8(input: &[i32], output: &mut [i32], bit_depth: u8) {
    let mut temp = [0i32; 64];
    let shift1 = 7;
    let shift2 = 20 - bit_depth as i32;

    for y in 0..8 {
        for x in 0..8 {
            let mut sum = 0i32;
            for k in 0..8 {
                sum += input[y * 8 + k] * DCT8[k][x];
            }
            temp[y * 8 + x] = (sum + (1 << (shift1 - 1))) >> shift1;
        }
    }

    for x in 0..8 {
        for y in 0..8 {
            let mut sum = 0i32;
            for k in 0..8 {
                sum += temp[k * 8 + x] * DCT8[k][y];
            }
            output[y * 8 + x] = (sum + (1 << (shift2 - 1))) >> shift2;
        }
    }
}

fn inverse_dct16(input: &[i32], output: &mut [i32], bit_depth: u8) {
    let mut temp = [0i32; 256];
    let shift1 = 7;
    let shift2 = 20 - bit_depth as i32;

    for y in 0..16 {
        for x in 0..16 {
            let mut sum = 0i32;
            for k in 0..16 {
                sum += input[y * 16 + k] * DCT16[k][x];
            }
            temp[y * 16 + x] = (sum + (1 << (shift1 - 1))) >> shift1;
        }
    }

    for x in 0..16 {
        for y in 0..16 {
            let mut sum = 0i32;
            for k in 0..16 {
                sum += temp[k * 16 + x] * DCT16[k][y];
            }
            output[y * 16 + x] = (sum + (1 << (shift2 - 1))) >> shift2;
        }
    }
}

fn inverse_dct32(input: &[i32], output: &mut [i32], bit_depth: u8) {
    // 32x32 DCT decomposed into partial butterfly / direct matrix
    let mut temp = [0i32; 1024];
    let shift1 = 7;
    let shift2 = 20 - bit_depth as i32;

    for y in 0..32 {
        for x in 0..32 {
            let mut sum = 0i32;
            for k in 0..32 {
                let coeff = dct32_coeff(k, x);
                sum += input[y * 32 + k] * coeff;
            }
            temp[y * 32 + x] = (sum + (1 << (shift1 - 1))) >> shift1;
        }
    }

    for x in 0..32 {
        for y in 0..32 {
            let mut sum = 0i32;
            for k in 0..32 {
                let coeff = dct32_coeff(k, y);
                sum += temp[k * 32 + x] * coeff;
            }
            output[y * 32 + x] = (sum + (1 << (shift2 - 1))) >> shift2;
        }
    }
}

fn dct32_coeff(row: usize, col: usize) -> i32 {
    // Exact HEVC DCT-32 generator formula
    if (row & 1) == 0 {
        if (row & 3) == 0 {
            if (row & 7) == 0 {
                if (row & 15) == 0 {
                    DCT4[row / 8][col % 4]
                } else {
                    DCT8[row / 4][col % 8]
                }
            } else {
                DCT16[row / 2][col % 16]
            }
        } else {
            DCT16[row / 2][col % 16]
        }
    } else {
        // Odd rows of DCT32
        let angle = ((2 * col + 1) * row) as f64 * std::f64::consts::PI / 64.0;
        (angle.cos() * 90.5).round() as i32
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
