//! Planar (Mode 0) and DC (Mode 1) intra prediction.

use super::filtering::IntraReferences;

/// Computes Planar intra prediction (Section 8.4.4.2.5).
#[allow(clippy::needless_range_loop)]
pub fn predict_planar(refs: &IntraReferences, output: &mut [u16], stride: usize) {
    let n = refs.size;
    let log2_n = match n {
        4 => 2,
        8 => 3,
        16 => 4,
        32 => 5,
        _ => 2,
    };

    let top_right = refs.top[n + 1] as i32;
    let bottom_left = refs.left[n + 1] as i32;

    for y in 0..n {
        let left_val = refs.left[y + 1] as i32;
        for x in 0..n {
            let top_val = refs.top[x + 1] as i32;
            let val = (n - 1 - x) as i32 * left_val
                + (x + 1) as i32 * top_right
                + (n - 1 - y) as i32 * top_val
                + (y + 1) as i32 * bottom_left
                + n as i32;
            output[y * stride + x] = (val >> (log2_n + 1)) as u16;
        }
    }
}

/// Computes DC intra prediction with optional boundary filtering (Section 8.4.4.2.6).
#[allow(clippy::needless_range_loop)]
pub fn predict_dc(refs: &IntraReferences, output: &mut [u16], stride: usize, is_luma: bool) {
    let n = refs.size;
    let log2_n = match n {
        4 => 2,
        8 => 3,
        16 => 4,
        32 => 5,
        _ => 2,
    };

    let mut sum = 0u32;
    for i in 1..=n {
        sum += refs.top[i] as u32 + refs.left[i] as u32;
    }
    let dc_val = ((sum + (n as u32)) >> (log2_n + 1)) as u16;

    for y in 0..n {
        for x in 0..n {
            output[y * stride + x] = dc_val;
        }
    }

    // Boundary filtering for DC luma when size < 32
    if is_luma && n < 32 {
        output[0] =
            ((refs.left[1] as u32 + 2 * (dc_val as u32) + refs.top[1] as u32 + 2) >> 2) as u16;
        for x in 1..n {
            output[x] = ((refs.top[x + 1] as u32 + 3 * (dc_val as u32) + 2) >> 2) as u16;
        }
        for y in 1..n {
            output[y * stride] = ((refs.left[y + 1] as u32 + 3 * (dc_val as u32) + 2) >> 2) as u16;
        }
    }
}
