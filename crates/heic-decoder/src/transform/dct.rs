//! Discrete Cosine Transform (DCT-II) core matrices and 4x4, 8x8, 16x16, 32x32 inverse transforms.

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

/// Applies 2D Inverse DCT-II to 4x4 block.
pub fn inverse_dct4(input: &[i32], output: &mut [i32], bit_depth: u8) {
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

/// Applies 2D Inverse DCT-II to 8x8 block.
pub fn inverse_dct8(input: &[i32], output: &mut [i32], bit_depth: u8) {
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

/// Applies 2D Inverse DCT-II to 16x16 block.
pub fn inverse_dct16(input: &[i32], output: &mut [i32], bit_depth: u8) {
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

/// Computes 32x32 DCT coefficient.
pub fn dct32_coeff(row: usize, col: usize) -> i32 {
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
        let angle = ((2 * col + 1) * row) as f64 * std::f64::consts::PI / 64.0;
        (angle.cos() * 90.5).round() as i32
    }
}

/// Applies 2D Inverse DCT-II to 32x32 block.
pub fn inverse_dct32(input: &[i32], output: &mut [i32], bit_depth: u8) {
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
