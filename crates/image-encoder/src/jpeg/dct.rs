//! 2D Forward Discrete Cosine Transform (FDCT).

/// Encodes an 8x8 block with 2D Forward DCT (FDCT).
pub fn fdct_8x8(input: &[f32; 64], output: &mut [f32; 64]) {
    let mut temp = [0f32; 64];

    // Horizontal 1D DCT
    for y in 0..8 {
        for u in 0..8 {
            let cu = if u == 0 {
                1.0 / std::f32::consts::SQRT_2
            } else {
                1.0
            };
            let mut sum = 0.0f32;
            for x in 0..8 {
                let angle = (2 * x + 1) as f32 * u as f32 * std::f32::consts::PI / 16.0;
                sum += input[y * 8 + x] * angle.cos();
            }
            temp[y * 8 + u] = 0.5 * cu * sum;
        }
    }

    // Vertical 1D DCT
    for u in 0..8 {
        for v in 0..8 {
            let cv = if v == 0 {
                1.0 / std::f32::consts::SQRT_2
            } else {
                1.0
            };
            let mut sum = 0.0f32;
            for y in 0..8 {
                let angle = (2 * y + 1) as f32 * v as f32 * std::f32::consts::PI / 16.0;
                sum += temp[y * 8 + u] * angle.cos();
            }
            output[v * 8 + u] = 0.5 * cv * sum;
        }
    }
}
