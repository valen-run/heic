//! Picture Parameter Set (PPS) parser.

use crate::nal::bit_reader::BitReader;
use valen_heic_core::HeicResult;

/// Parsed Picture Parameter Set (`PPS`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pps {
    /// PPS identifier (`pps_pic_parameter_set_id`).
    pub pps_id: u32,
    /// Linked SPS identifier.
    pub sps_id: u32,
    /// Whether dependent slice segments are enabled.
    pub dependent_slice_segments_enabled: bool,
    /// Whether output flag is present in slice header.
    pub output_flag_present: bool,
    /// Number of extra slice header bits.
    pub num_extra_slice_header_bits: u8,
    /// Whether sign data hiding is enabled.
    pub sign_data_hiding_enabled: bool,
    /// Whether CABAC init flag is present in slice headers.
    pub cabac_init_present: bool,
    /// Initial QP minus 26.
    pub init_qp_minus26: i32,
    /// Whether constrained intra prediction is used.
    pub constrained_intra_pred: bool,
    /// Whether transform skip is enabled.
    pub transform_skip_enabled: bool,
    /// Whether CU QP delta is enabled.
    pub cu_qp_delta_enabled: bool,
    /// Difference of CU QP delta depth.
    pub diff_cu_qp_delta_depth: u32,
    /// Chroma Cb QP offset.
    pub pps_cb_qp_offset: i32,
    /// Chroma Cr QP offset.
    pub pps_cr_qp_offset: i32,
    /// Whether slice chroma QP offsets are present in slice headers.
    pub slice_chroma_qp_offsets_present: bool,
    /// Whether deblocking filter control is present in PPS.
    pub deblocking_filter_control_present: bool,
    /// Whether deblocking filter is disabled by default in PPS.
    pub deblocking_filter_disabled: bool,
    /// Beta offset divided by 2.
    pub beta_offset_div2: i32,
    /// Tc offset divided by 2.
    pub tc_offset_div2: i32,
}

impl Pps {
    /// Parses a PPS from RBSP bytes.
    pub fn parse(rbsp: &[u8]) -> HeicResult<Self> {
        let mut r = BitReader::new(rbsp);

        let pps_id = r.read_ue()?;
        let sps_id = r.read_ue()?;
        let dependent_slice_segments_enabled = r.read_bit()? != 0;
        let output_flag_present = r.read_bit()? != 0;
        let num_extra_slice_header_bits = r.read_bits(3)? as u8;
        let sign_data_hiding_enabled = r.read_bit()? != 0;
        let cabac_init_present = r.read_bit()? != 0;
        let _num_ref_idx_l0_default_active_minus1 = r.read_ue()?;
        let _num_ref_idx_l1_default_active_minus1 = r.read_ue()?;
        let init_qp_minus26 = r.read_se()?;
        let constrained_intra_pred = r.read_bit()? != 0;
        let transform_skip_enabled = r.read_bit()? != 0;

        let cu_qp_delta_enabled = r.read_bit()? != 0;
        let diff_cu_qp_delta_depth = if cu_qp_delta_enabled { r.read_ue()? } else { 0 };

        let pps_cb_qp_offset = r.read_se()?;
        let pps_cr_qp_offset = r.read_se()?;
        let slice_chroma_qp_offsets_present = r.read_bit()? != 0;
        let _weighted_pred_flag = r.read_bit()?;
        let _weighted_bipred_flag = r.read_bit()?;
        let _transquant_bypass_enabled_flag = r.read_bit()?;
        let tiles_enabled_flag = r.read_bit()? != 0;
        let _entropy_coding_sync_enabled_flag = r.read_bit()?;

        if tiles_enabled_flag {
            let num_tile_columns_minus1 = r.read_ue()?;
            let num_tile_rows_minus1 = r.read_ue()?;
            let uniform_spacing_flag = r.read_bit()? != 0;
            if !uniform_spacing_flag {
                for _ in 0..num_tile_columns_minus1 {
                    let _column_width_minus1 = r.read_ue()?;
                }
                for _ in 0..num_tile_rows_minus1 {
                    let _row_height_minus1 = r.read_ue()?;
                }
            }
            let _loop_filter_across_tiles_enabled_flag = r.read_bit()?;
        }

        let _pps_loop_filter_across_slices_enabled_flag = r.read_bit()?;
        let deblocking_filter_control_present = r.read_bit()? != 0;
        let mut deblocking_filter_disabled = false;
        let mut beta_offset_div2 = 0;
        let mut tc_offset_div2 = 0;

        if deblocking_filter_control_present {
            let deblocking_filter_override_enabled_flag = r.read_bit()? != 0;
            let pps_deblocking_filter_disabled_flag = r.read_bit()? != 0;
            deblocking_filter_disabled = pps_deblocking_filter_disabled_flag;
            if !pps_deblocking_filter_disabled_flag {
                beta_offset_div2 = r.read_se()?;
                tc_offset_div2 = r.read_se()?;
            }
            let _ = deblocking_filter_override_enabled_flag;
        }

        Ok(Self {
            pps_id,
            sps_id,
            dependent_slice_segments_enabled,
            output_flag_present,
            num_extra_slice_header_bits,
            sign_data_hiding_enabled,
            cabac_init_present,
            init_qp_minus26,
            constrained_intra_pred,
            transform_skip_enabled,
            cu_qp_delta_enabled,
            diff_cu_qp_delta_depth,
            pps_cb_qp_offset,
            pps_cr_qp_offset,
            slice_chroma_qp_offsets_present,
            deblocking_filter_control_present,
            deblocking_filter_disabled,
            beta_offset_div2,
            tc_offset_div2,
        })
    }
}
