//! HEVC Slice Segment Header parser.

use crate::nal::bit_reader::BitReader;
use crate::nal::pps::Pps;
use crate::nal::sps::Sps;
use crate::nal::unit::NalUnitType;
use valen_heic_core::{HeicError, HeicResult};

/// Slice type identifier (ITU-T H.265 Table 7-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceType {
    /// B-slice (bi-predictive).
    B = 0,
    /// P-slice (predictive).
    P = 1,
    /// I-slice (intra).
    I = 2,
}

/// Parsed HEVC slice segment header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceHeader {
    /// First slice segment in picture flag.
    pub first_slice_segment_in_pic_flag: bool,
    /// Linked PPS identifier.
    pub pps_id: u32,
    /// Slice type (I, P, or B).
    pub slice_type: SliceType,
    /// Computed initial slice quantization parameter (QP).
    pub slice_qp: i32,
    /// Luma SAO enabled in slice.
    pub slice_sao_luma_flag: bool,
    /// Chroma SAO enabled in slice.
    pub slice_sao_chroma_flag: bool,
    /// Deblocking filter disabled for this slice.
    pub slice_deblocking_filter_disabled: bool,
    /// Slice beta offset divided by 2.
    pub slice_beta_offset_div2: i32,
    /// Slice tc offset divided by 2.
    pub slice_tc_offset_div2: i32,
}

impl SliceHeader {
    /// Parses a slice segment header given NAL unit properties, SPS, and PPS.
    pub fn parse(
        rbsp: &[u8],
        nal_unit_type: NalUnitType,
        sps: &Sps,
        pps: &Pps,
    ) -> HeicResult<(Self, usize, u8)> {
        let mut r = BitReader::new(rbsp);

        let first_slice_segment_in_pic_flag = r.read_bit()? != 0;
        if nal_unit_type.is_irap() {
            let _no_output_of_prior_pics_flag = r.read_bit()?;
        }

        let pps_id = r.read_ue()?;
        if pps_id != pps.pps_id {
            return Err(HeicError::InvalidContainer(format!(
                "Slice references PPS ID {pps_id} but active PPS ID is {}",
                pps.pps_id
            )));
        }

        let slice_type_val = if !first_slice_segment_in_pic_flag {
            // Dependent slices or non-first slices
            if pps.dependent_slice_segments_enabled {
                let dependent_slice_segment_flag = r.read_bit()? != 0;
                if dependent_slice_segment_flag {
                    // Inherits previous slice header settings
                    return Ok((
                        Self {
                            first_slice_segment_in_pic_flag,
                            pps_id,
                            slice_type: SliceType::I,
                            slice_qp: 26 + pps.init_qp_minus26,
                            slice_sao_luma_flag: sps.sample_adaptive_offset_enabled,
                            slice_sao_chroma_flag: sps.sample_adaptive_offset_enabled,
                            slice_deblocking_filter_disabled: pps.deblocking_filter_disabled,
                            slice_beta_offset_div2: pps.beta_offset_div2,
                            slice_tc_offset_div2: pps.tc_offset_div2,
                        },
                        r.byte_offset,
                        r.bit_offset,
                    ));
                }
            }
            let ctu_count = sps.pic_width_in_luma_samples.div_ceil(sps.ctu_size())
                * sps.pic_height_in_luma_samples.div_ceil(sps.ctu_size());
            let bits = 32 - ctu_count.saturating_sub(1).leading_zeros();
            let _slice_segment_address = r.read_bits(bits as u8)?;
            r.read_ue()?
        } else {
            r.read_ue()?
        };

        let slice_type = match slice_type_val {
            0 => SliceType::B,
            1 => SliceType::P,
            2 => SliceType::I,
            _ => SliceType::I,
        };

        if pps.output_flag_present {
            let _pic_output_flag = r.read_bit()?;
        }

        let mut slice_sao_luma_flag = false;
        let mut slice_sao_chroma_flag = false;
        if sps.sample_adaptive_offset_enabled {
            slice_sao_luma_flag = r.read_bit()? != 0;
            slice_sao_chroma_flag = r.read_bit()? != 0;
        }

        let slice_qp_delta = r.read_se()?;
        let slice_qp = 26 + pps.init_qp_minus26 + slice_qp_delta;

        let mut slice_deblocking_filter_disabled = pps.deblocking_filter_disabled;
        let mut slice_beta_offset_div2 = pps.beta_offset_div2;
        let mut slice_tc_offset_div2 = pps.tc_offset_div2;

        if pps.deblocking_filter_control_present {
            let deblocking_filter_override_flag = r.read_bit()? != 0;
            if deblocking_filter_override_flag {
                slice_deblocking_filter_disabled = r.read_bit()? != 0;
                if !slice_deblocking_filter_disabled {
                    slice_beta_offset_div2 = r.read_se()?;
                    slice_tc_offset_div2 = r.read_se()?;
                }
            }
        }

        // Align reader to byte boundary for CABAC initialization
        r.byte_align();

        Ok((
            Self {
                first_slice_segment_in_pic_flag,
                pps_id,
                slice_type,
                slice_qp,
                slice_sao_luma_flag,
                slice_sao_chroma_flag,
                slice_deblocking_filter_disabled,
                slice_beta_offset_div2,
                slice_tc_offset_div2,
            },
            r.byte_offset,
            r.bit_offset,
        ))
    }
}
