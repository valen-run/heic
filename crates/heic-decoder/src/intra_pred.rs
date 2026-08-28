//! 35 HEVC Intra Prediction modes and reference sample filtering (ITU-T H.265 Section 8.4).

/// Directional intra prediction angle lookup table for angular modes 2..=34.
///
/// In HEVC, intra prediction angles represent 1/32th sub-pel displacement steps:
/// - Modes 2..17: Horizontal-like angles predicting from left reference column
/// - Mode 18..34: Vertical-like angles predicting from top reference row
/// - Mode 10 is pure horizontal (angle = 0), Mode 26 is pure vertical (angle = 0)
pub static INTRA_PRED_ANGLE: [i32; 33] = [
    32, 26, 21, 17, 13, 9, 5, 2, 0, -2, -5, -9, -13, -17, -21, -26, -32, -26, -21, -17, -13, -9,
    -5, -2, 0, 2, 5, 9, 13, 17, 21, 26, 32,
];

/// Inverse angle lookup table for negative angles to project top/left reference samples across the corner.
///
/// Scaled by 8192 / angle according to Section 8.4.4.2.7.
pub static INV_ANGLE: [i32; 33] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, -4096, -1638, -910, -630, -482, -390, -315, -256, -315, -390, -482,
    -630, -910, -1638, -4096, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Boundary and neighboring reference samples for an intra prediction block of size `N x N`.
#[derive(Debug, Clone)]
pub struct IntraReferences {
    /// Reference samples: index 0 is top-left `(-1, -1)`,
    /// indices `1..=2*N` are top and top-right `(0..2N-1, -1)`.
    pub top: Vec<u16>,
    /// Reference samples: index 0 is top-left `(-1, -1)`,
    /// indices `1..=2*N` are left and bottom-left `(-1, 0..2N-1)`.
    pub left: Vec<u16>,
    /// Block size `N` (e.g. 4, 8, 16, 32).
    pub size: usize,
}

impl IntraReferences {
    /// Creates a new reference sample container for block size `N` initialized with DC default.
    pub fn new(size: usize, default_val: u16) -> Self {
        Self {
            top: vec![default_val; 2 * size + 1],
            left: vec![default_val; 2 * size + 1],
            size,
        }
    }

    /// Prepares smoothed reference samples according to Section 8.4.4.2.3.
    ///
    /// Applies conditional 3-tap reference smoothing filter `[1, 2, 1]/4` or Strong Intra Smoothing (for 32x32 blocks).
    pub fn filter_references(&self, mode: u8, is_luma: bool, bit_depth: u8) -> Self {
        let n = self.size;
        let mut filtered_top = self.top.clone();
        let mut filtered_left = self.left.clone();

        if !is_luma || n == 4 {
            return self.clone();
        }

        // Intra smoothing filter check
        let smoothing_needed = match n {
            8 => mode == 0 || mode == 2 || ((18..=34).contains(&mode) && ((mode - 18) & 3) == 0),
            16 => mode != 9 && mode != 10 && mode != 11 && mode != 25 && mode != 26 && mode != 27,
            32 => mode != 10 && mode != 26,
            _ => false,
        };

        if !smoothing_needed {
            return self.clone();
        }

        // Strong intra smoothing check for 32x32 blocks
        if n == 32 {
            let top_left = self.top[0] as i32;
            let top_end = self.top[2 * n] as i32;
            let left_end = self.left[2 * n] as i32;
            let threshold = 1 << (bit_depth - 5);

            let cond1 = (top_left - 2 * (self.top[n] as i32) + top_end).abs() < threshold;
            let cond2 = (top_left - 2 * (self.left[n] as i32) + left_end).abs() < threshold;

            if cond1 && cond2 {
                for i in 1..=2 * n {
                    filtered_top[i] =
                        (((2 * n - i) as i32 * top_left + i as i32 * top_end + 32) >> 6) as u16;
                    filtered_left[i] =
                        (((2 * n - i) as i32 * top_left + i as i32 * left_end + 32) >> 6) as u16;
                }
                return Self {
                    top: filtered_top,
                    left: filtered_left,
                    size: n,
                };
            }
        }

        // Standard 3-tap filter [1, 2, 1] / 4
        filtered_top[0] =
            ((self.left[1] as u32 + 2 * self.top[0] as u32 + self.top[1] as u32 + 2) >> 2) as u16;
        filtered_left[0] = filtered_top[0];

        for i in 1..2 * n {
            filtered_top[i] =
                ((self.top[i - 1] as u32 + 2 * self.top[i] as u32 + self.top[i + 1] as u32 + 2)
                    >> 2) as u16;
            filtered_left[i] =
                ((self.left[i - 1] as u32 + 2 * self.left[i] as u32 + self.left[i + 1] as u32 + 2)
                    >> 2) as u16;
        }

        Self {
            top: filtered_top,
            left: filtered_left,
            size: n,
        }
    }
}

/// Generates predicted samples for an `N x N` block using the given intra mode (0..=34).
#[allow(clippy::needless_range_loop)]
pub fn predict_intra(
    mode: u8,
    refs: &IntraReferences,
    output: &mut [u16],
    stride: usize,
    is_luma: bool,
) {
    let n = refs.size;
    let log2_n = match n {
        4 => 2,
        8 => 3,
        16 => 4,
        32 => 5,
        _ => 2,
    };

    match mode {
        0 => {
            // Planar mode (Section 8.4.4.2.5)
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
        1 => {
            // DC mode (Section 8.4.4.2.6)
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
                output[0] = ((refs.left[1] as u32 + 2 * (dc_val as u32) + refs.top[1] as u32 + 2)
                    >> 2) as u16;
                for x in 1..n {
                    output[x] = ((refs.top[x + 1] as u32 + 3 * (dc_val as u32) + 2) >> 2) as u16;
                }
                for y in 1..n {
                    output[y * stride] =
                        ((refs.left[y + 1] as u32 + 3 * (dc_val as u32) + 2) >> 2) as u16;
                }
            }
        }
        2..=34 => {
            // Angular modes (Section 8.4.4.2.7)
            let angle_idx = (mode - 2) as usize;
            let intra_pred_angle = INTRA_PRED_ANGLE[angle_idx];
            let inv_angle = INV_ANGLE[angle_idx];

            let mut ref_main = vec![0i32; 4 * n + 1];

            if mode >= 18 {
                // Vertical-like modes
                for i in 0..=2 * n {
                    ref_main[2 * n + i] = refs.top[i] as i32;
                }
                if intra_pred_angle < 0 {
                    let max_idx = (n as i32 * intra_pred_angle) >> 5;
                    let mut x = -1;
                    while x >= max_idx {
                        let idx = (x * inv_angle + 128) >> 8;
                        if idx >= 0 && (idx as usize) <= 2 * n {
                            ref_main[2 * n + (x as isize) as usize] =
                                refs.left[idx as usize] as i32;
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
                        let val = output[x] as i32
                            + ((refs.top[x + 1] as i32 - refs.left[0] as i32) >> 1);
                        output[x] = val.clamp(0, 65535) as u16;
                    }
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planar_prediction() {
        let refs = IntraReferences::new(4, 100);
        let mut out = vec![0u16; 16];
        predict_intra(0, &refs, &mut out, 4, true);
        for &val in &out {
            assert_eq!(val, 100);
        }
    }

    #[test]
    fn test_dc_prediction() {
        let refs = IntraReferences::new(4, 128);
        let mut out = vec![0u16; 16];
        predict_intra(1, &refs, &mut out, 4, false);
        for &val in &out {
            assert_eq!(val, 128);
        }
    }

    #[test]
    fn test_vertical_prediction() {
        let mut refs = IntraReferences::new(4, 0);
        for i in 1..=8 {
            refs.top[i] = (i * 10) as u16;
        }
        let mut out = vec![0u16; 16];
        predict_intra(26, &refs, &mut out, 4, false);
        assert_eq!(out[0], 10);
        assert_eq!(out[1], 20);
        assert_eq!(out[2], 30);
        assert_eq!(out[3], 40);
        assert_eq!(out[4], 10);
    }
}
