//! 35 HEVC Intra Prediction modes and reference sample filtering (ITU-T H.265 Section 8.4).

pub mod angular;
pub mod filtering;
pub mod planar_dc;

pub use angular::{predict_angular, INTRA_PRED_ANGLE, INV_ANGLE};
pub use filtering::IntraReferences;
pub use planar_dc::{predict_dc, predict_planar};

/// Generates predicted samples for an `N x N` block using the given intra mode (0..=34).
pub fn predict_intra(
    mode: u8,
    refs: &IntraReferences,
    output: &mut [u16],
    stride: usize,
    is_luma: bool,
) {
    match mode {
        0 => predict_planar(refs, output, stride),
        1 => predict_dc(refs, output, stride, is_luma),
        2..=34 => predict_angular(mode, refs, output, stride, is_luma),
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
