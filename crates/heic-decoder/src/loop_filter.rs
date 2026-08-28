//! Deblocking filter and Sample Adaptive Offset (SAO) (ITU-T H.265 Section 8.7).

/// Beta threshold lookup table indexed by `QP`.
pub static BETA_TABLE: [u8; 52] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
    20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 58, 60, 62, 64,
];

/// Tc threshold lookup table indexed by `QP + (bS - 2) * 2`.
pub static TC_TABLE: [u8; 54] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3,
    3, 3, 3, 4, 4, 4, 5, 5, 6, 6, 7, 8, 9, 10, 11, 13, 14, 16, 18, 20, 22, 24,
];

/// Deblocking filter edge orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    /// Vertical edge (separates left and right samples).
    Vertical,
    /// Horizontal edge (separates top and bottom samples).
    Horizontal,
}

/// Applies HEVC in-loop deblocking filter on reconstructed frame plane (ITU-T H.265 Section 8.7.2).
///
/// Filters artifacts across 4-sample block boundaries:
/// - Evaluates boundary flatness condition `d < beta` where `d = |p2 - 2*p1 + p0| + |q2 - 2*q1 + q0|`
/// - Computes clipping limit `t_C` and delta adjustment `delta = ((9*(q0 - p0) - 3*(q1 - p1) + 8) >> 4).clamp(-tc, tc)`
/// - Updates `p0`, `q0`, and optionally `p1`, `q1` if local variance allows
#[allow(clippy::too_many_arguments)]
pub fn deblock_luma_edge(
    samples: &mut [u16],
    stride: usize,
    x: usize,
    y: usize,
    edge_type: EdgeType,
    qp: i32,
    beta_offset: i32,
    tc_offset: i32,
    bit_depth: u8,
) {
    let qp_clamped = (qp + beta_offset).clamp(0, 51) as usize;
    let beta = BETA_TABLE[qp_clamped] as i32 * (1 << (bit_depth - 8));

    let qp_tc_clamped = (qp + 2 + tc_offset).clamp(0, 53) as usize;
    let tc = TC_TABLE[qp_tc_clamped] as i32 * (1 << (bit_depth - 8));

    if tc == 0 || beta == 0 {
        return;
    }

    let max_val = (1 << bit_depth) - 1;

    for k in 0..4 {
        let (p0_idx, p1_idx, p2_idx, q0_idx, q1_idx, q2_idx) = match edge_type {
            EdgeType::Vertical => {
                let row = y + k;
                (
                    row * stride + x - 1,
                    row * stride + x - 2,
                    row * stride + x - 3,
                    row * stride + x,
                    row * stride + x + 1,
                    row * stride + x + 2,
                )
            }
            EdgeType::Horizontal => {
                let col = x + k;
                (
                    (y - 1) * stride + col,
                    (y - 2) * stride + col,
                    (y - 3) * stride + col,
                    y * stride + col,
                    (y + 1) * stride + col,
                    (y + 2) * stride + col,
                )
            }
        };

        let p0 = samples[p0_idx] as i32;
        let p1 = samples[p1_idx] as i32;
        let p2 = samples[p2_idx] as i32;
        let q0 = samples[q0_idx] as i32;
        let q1 = samples[q1_idx] as i32;
        let q2 = samples[q2_idx] as i32;

        let d_p = (p2 - 2 * p1 + p0).abs();
        let d_q = (q2 - 2 * q1 + q0).abs();
        let d = d_p + d_q;

        if d < beta {
            // Normal filter
            let delta = ((9 * (q0 - p0) - 3 * (q1 - p1) + 8) >> 4).clamp(-tc, tc);
            samples[p0_idx] = (p0 + delta).clamp(0, max_val) as u16;
            samples[q0_idx] = (q0 - delta).clamp(0, max_val) as u16;

            if d_p < ((beta + (beta >> 1)) >> 3) {
                let delta_p = (((p2 + p0 + 1) >> 1) - p1 + delta) >> 1;
                samples[p1_idx] =
                    (p1 + delta_p.clamp(-(tc >> 1), tc >> 1)).clamp(0, max_val) as u16;
            }
            if d_q < ((beta + (beta >> 1)) >> 3) {
                let delta_q = (((q2 + q0 + 1) >> 1) - q1 - delta) >> 1;
                samples[q1_idx] =
                    (q1 + delta_q.clamp(-(tc >> 1), tc >> 1)).clamp(0, max_val) as u16;
            }
        }
    }
}

/// Applies Sample Adaptive Offset (SAO) Edge Offset to a block of reconstructed samples (Section 8.7.5).
pub fn apply_sao_edge_offset(
    samples: &mut [u16],
    stride: usize,
    width: usize,
    height: usize,
    eo_class: u8,
    offsets: &[i32; 4],
    bit_depth: u8,
) {
    let max_val = (1 << bit_depth) - 1;

    let (dx0, dy0, dx1, dy1) = match eo_class {
        0 => (-1, 0, 1, 0),  // 0 deg horizontal
        1 => (0, -1, 0, 1),  // 90 deg vertical
        2 => (-1, -1, 1, 1), // 135 deg diagonal
        3 => (1, -1, -1, 1), // 45 deg diagonal
        _ => return,
    };

    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let cur_idx = y * stride + x;
            let p0_idx = ((y as isize + dy0) as usize) * stride + ((x as isize + dx0) as usize);
            let p1_idx = ((y as isize + dy1) as usize) * stride + ((x as isize + dx1) as usize);

            let c = samples[cur_idx] as i32;
            let a = samples[p0_idx] as i32;
            let b = samples[p1_idx] as i32;

            let cat = if c < a && c < b {
                1
            } else if (c < a && c == b) || (c == a && c < b) {
                2
            } else if (c > a && c == b) || (c == a && c > b) {
                3
            } else if c > a && c > b {
                4
            } else {
                0
            };

            if cat > 0 {
                let offset = match cat {
                    1 => offsets[0],
                    2 => offsets[1],
                    3 => -offsets[2],
                    4 => -offsets[3],
                    _ => 0,
                };
                samples[cur_idx] = (c + offset).clamp(0, max_val) as u16;
            }
        }
    }
}

/// Applies Sample Adaptive Offset (SAO) Band Offset (Section 8.7.5).
pub fn apply_sao_band_offset(
    samples: &mut [u16],
    stride: usize,
    width: usize,
    height: usize,
    start_band: u8,
    offsets: &[i32; 4],
    bit_depth: u8,
) {
    let max_val = (1 << bit_depth) - 1;
    let shift = bit_depth - 5;

    for y in 0..height {
        for x in 0..width {
            let idx = y * stride + x;
            let c = samples[idx] as i32;
            let band_idx = (c >> shift) as u8;

            if band_idx >= start_band && band_idx < start_band + 4 {
                let offset = offsets[(band_idx - start_band) as usize];
                samples[idx] = (c + offset).clamp(0, max_val) as u16;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sao_band_offset() {
        let mut samples = vec![128u16; 16];
        let offsets = [2, 4, 6, 8];
        let start_band = 128 >> 3; // band 16 for 8-bit (128 >> 3 = 16)
        apply_sao_band_offset(&mut samples, 4, 4, 4, start_band, &offsets, 8);
        for &s in &samples {
            assert_eq!(s, 130);
        }
    }
}
