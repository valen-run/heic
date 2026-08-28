//! 4x4 Discrete Sine Transform (DST-VII) for HEVC 4x4 intra luma residuals.

/// 4x4 DST-VII core transform matrix (for 4x4 intra luma residual).
pub static DST4: [[i32; 4]; 4] = [
    [29, 55, 74, 84],
    [74, 74, 0, -74],
    [84, -29, -74, 55],
    [55, -84, 74, -29],
];

/// Applies 2D Inverse DST-VII to 4x4 transform block.
pub fn inverse_dst4(input: &[i32], output: &mut [i32], bit_depth: u8) {
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
