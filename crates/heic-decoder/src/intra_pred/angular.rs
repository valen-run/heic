//! Angular intra prediction modes 2..=34 (ITU-T H.265 Section 8.4.4.2.7).

use super::filtering::IntraReferences;

/// Directional intra prediction angle lookup table for angular modes 2..=34.
pub static INTRA_PRED_ANGLE: [i32; 33] = [
    32, 26, 21, 17, 13, 9, 5, 2, 0, -2, -5, -9, -13, -17, -21, -26, -32, -26, -21, -17, -13, -9,
    -5, -2, 0, 2, 5, 9, 13, 17, 21, 26, 32,
];

/// Inverse angle lookup table for negative angles to project top/left reference samples across the corner.
pub static INV_ANGLE: [i32; 33] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, -4096, -1638, -910, -630, -482, -390, -315, -256, -315, -390, -482,
    -630, -910, -1638, -4096, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Computes Angular intra prediction for modes 2..=34.
#[allow(clippy::needless_range_loop)]
pub fn predict_angular(
    mode: u8,
    refs: &IntraReferences,
    output: &mut [u16],
    stride: usize,
    is_luma: bool,
) {
    let n = refs.size;
    let angle_idx = (mode - 2) as usize;
    let intra_pred_angle = INTRA_PRED_ANGLE[angle_idx];
    let inv_angle = INV_ANGLE[angle_idx];

    let mut ref_main = vec![0i32; 4 * n + 1];

    if mode >= 18 {
        // Vertical-like modes (18..=34)
        for i in 0..=2 * n {
            ref_main[2 * n + i] = refs.top[i] as i32;
        }
        if intra_pred_angle < 0 {
            let max_idx = (n as i32 * intra_pred_angle) >> 5;
            let mut x = -1;
            while x >= max_idx {
                let idx = (x * inv_angle + 128) >> 8;
                if idx >= 0 && (idx as usize) <= 2 * n {
                    ref_main[2 * n + (x as isize) as usize] = refs.left[idx as usize] as i32;
                }
                x -= 1;
            }
        }

        for y in 0..n {
            let delta_pos = (y as i32 + 1) * intra_pred_angle;
            let delta_int = delta_pos >> 5;
            let delta_fract = delta_pos & 31;

            for x in 0..n {
                let ref_idx = (2 * n as i32 + x as i32 + delta_int + 1) as usize;
                if delta_fract != 0 {
                    let val = (32 - delta_fract) * ref_main[ref_idx]
                        + delta_fract * ref_main[ref_idx + 1]
                        + 16;
                    output[y * stride + x] = (val >> 5) as u16;
                } else {
                    output[y * stride + x] = ref_main[ref_idx] as u16;
                }
            }
        }

        // Boundary filter for Mode 26 (pure vertical)
        if is_luma && mode == 26 && n < 32 {
            for y in 0..n {
                let val = output[y * stride] as i32
                    + ((refs.left[y + 1] as i32 - refs.top[0] as i32) >> 1);
                output[y * stride] = val.clamp(0, 65535) as u16;
            }
        }
    } else {
        // Horizontal-like modes (2..=17)
        for i in 0..=2 * n {
            ref_main[2 * n + i] = refs.left[i] as i32;
        }
        if intra_pred_angle < 0 {
            let max_idx = (n as i32 * intra_pred_angle) >> 5;
            let mut y = -1;
            while y >= max_idx {
                let idx = (y * inv_angle + 128) >> 8;
                if idx >= 0 && (idx as usize) <= 2 * n {
                    ref_main[2 * n + (y as isize) as usize] = refs.top[idx as usize] as i32;
                }
                y -= 1;
            }
        }

        for y in 0..n {
            for x in 0..n {
                let delta_pos = (x as i32 + 1) * intra_pred_angle;
                let delta_int = delta_pos >> 5;
                let delta_fract = delta_pos & 31;
                let ref_idx = (2 * n as i32 + y as i32 + delta_int + 1) as usize;

                if delta_fract != 0 {
                    let val = (32 - delta_fract) * ref_main[ref_idx]
                        + delta_fract * ref_main[ref_idx + 1]
                        + 16;
                    output[y * stride + x] = (val >> 5) as u16;
                } else {
                    output[y * stride + x] = ref_main[ref_idx] as u16;
                }
            }
        }

        // Boundary filter for Mode 10 (pure horizontal)
        if is_luma && mode == 10 && n < 32 {
            for x in 0..n {
                let val = output[x] as i32 + ((refs.top[x + 1] as i32 - refs.left[0] as i32) >> 1);
                output[x] = val.clamp(0, 65535) as u16;
            }
        }
    }
}
