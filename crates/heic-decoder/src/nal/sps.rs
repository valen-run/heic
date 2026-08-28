//! Sequence Parameter Set (SPS) parser.

use crate::nal::bit_reader::BitReader;
use valen_heic_core::{HeicResult, Limits};

/// Parsed Sequence Parameter Set (`SPS`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sps {
    /// SPS identifier (`sps_seq_parameter_set_id`).
    pub sps_id: u32,
    /// Chroma format IDC (0 = Monochrome, 1 = 4:2:0, 2 = 4:2:2, 3 = 4:4:4).
    pub chroma_format_idc: u32,
    /// Luma bit depth (e.g. 8 or 10).
    pub bit_depth_luma: u8,
    /// Chroma bit depth (e.g. 8 or 10).
    pub bit_depth_chroma: u8,
    /// Picture width in luma samples.
    pub pic_width_in_luma_samples: u32,
    /// Picture height in luma samples.
    pub pic_height_in_luma_samples: u32,
    /// Log2 of minimum Coding Block size (typically 3 for 8x8).
    pub log2_min_cb_size: u8,
    /// Difference between max and min CB size (CTU size = 1 << (log2_min_cb_size + diff)).
    pub log2_diff_max_min_cb_size: u8,
    /// Log2 of minimum Transform Block size (typically 2 for 4x4).
    pub log2_min_tb_size: u8,
    /// Difference between max and min TB size (max TB size = 1 << (log2_min_tb_size + diff)).
    pub log2_diff_max_min_tb_size: u8,
    /// Max transform hierarchy depth for intra coding units.
    pub max_transform_hierarchy_depth_intra: u32,
    /// Whether scaling list is enabled.
    pub scaling_list_enabled: bool,
    /// Whether Sample Adaptive Offset (SAO) is enabled.
    pub sample_adaptive_offset_enabled: bool,
    /// Whether PCM is enabled.
    pub pcm_enabled: bool,
}

impl Sps {
    /// CTU (Coding Tree Unit) size in luma samples (e.g. 16, 32, 64).
    pub const fn ctu_size(&self) -> u32 {
        1 << (self.log2_min_cb_size + self.log2_diff_max_min_cb_size)
    }

    /// Minimum coding block size (e.g. 8).
    pub const fn min_cb_size(&self) -> u32 {
        1 << self.log2_min_cb_size
    }

    /// Max transform block size (e.g. 32).
    pub const fn max_tb_size(&self) -> u32 {
        1 << (self.log2_min_tb_size + self.log2_diff_max_min_tb_size)
    }

    /// Min transform block size (e.g. 4).
    pub const fn min_tb_size(&self) -> u32 {
        1 << self.log2_min_tb_size
    }

    /// Parses an SPS from RBSP bytes.
    pub fn parse(rbsp: &[u8], limits: &Limits) -> HeicResult<Self> {
        let mut r = BitReader::new(rbsp);

        let _sps_video_parameter_set_id = r.read_bits(4)?;
        let sps_max_sub_layers_minus1 = r.read_bits(3)?;
        let _sps_temporal_id_nesting_flag = r.read_bit()?;

        // profile_tier_level
        parse_profile_tier_level(&mut r, sps_max_sub_layers_minus1 as u8)?;

        let sps_id = r.read_ue()?;
        let chroma_format_idc = r.read_ue()?;
        if chroma_format_idc == 3 {
            let _separate_colour_plane_flag = r.read_bit()?;
        }

        let pic_width_in_luma_samples = r.read_ue()?;
        let pic_height_in_luma_samples = r.read_ue()?;

        limits.check_dimensions(valen_heic_core::ImageDimensions::new(
            pic_width_in_luma_samples,
            pic_height_in_luma_samples,
        ))?;

        let conformance_window_flag = r.read_bit()? != 0;
        if conformance_window_flag {
            let _conf_win_left_offset = r.read_ue()?;
            let _conf_win_right_offset = r.read_ue()?;
            let _conf_win_top_offset = r.read_ue()?;
            let _conf_win_bottom_offset = r.read_ue()?;
        }

        let bit_depth_luma_minus8 = r.read_ue()?;
        let bit_depth_chroma_minus8 = r.read_ue()?;
        let bit_depth_luma = (8 + bit_depth_luma_minus8) as u8;
        let bit_depth_chroma = (8 + bit_depth_chroma_minus8) as u8;

        let _log2_max_pic_order_cnt_lsb_minus4 = r.read_ue()?;
        let sps_sub_layer_ordering_info_present_flag = r.read_bit()? != 0;

        let start_sub_layer = if sps_sub_layer_ordering_info_present_flag {
            0
        } else {
            sps_max_sub_layers_minus1
        };

        for _ in start_sub_layer..=sps_max_sub_layers_minus1 {
            let _sps_max_dec_pic_buffering_minus1 = r.read_ue()?;
            let _sps_max_num_reorder_pics = r.read_ue()?;
            let _sps_max_latency_increase_plus1 = r.read_ue()?;
        }

        let log2_min_luma_coding_block_size_minus3 = r.read_ue()?;
        let log2_diff_max_min_luma_coding_block_size = r.read_ue()?;
        let log2_min_luma_transform_block_size_minus2 = r.read_ue()?;
        let log2_diff_max_min_luma_transform_block_size = r.read_ue()?;

        let log2_min_cb_size = (3 + log2_min_luma_coding_block_size_minus3) as u8;
        let log2_diff_max_min_cb_size = log2_diff_max_min_luma_coding_block_size as u8;
        let log2_min_tb_size = (2 + log2_min_luma_transform_block_size_minus2) as u8;
        let log2_diff_max_min_tb_size = log2_diff_max_min_luma_transform_block_size as u8;

        let max_transform_hierarchy_depth_intra = r.read_ue()?;
        let _max_transform_hierarchy_depth_inter = r.read_ue()?;

        let scaling_list_enabled = r.read_bit()? != 0;
        if scaling_list_enabled {
            let sps_scaling_list_data_present_flag = r.read_bit()? != 0;
            if sps_scaling_list_data_present_flag {
                skip_scaling_list_data(&mut r)?;
            }
        }

        let _amp_enabled_flag = r.read_bit()?;
        let sample_adaptive_offset_enabled = r.read_bit()? != 0;

        let pcm_enabled = r.read_bit()? != 0;
        if pcm_enabled {
            let _pcm_sample_bit_depth_luma_minus1 = r.read_bits(4)?;
            let _pcm_sample_bit_depth_chroma_minus1 = r.read_bits(4)?;
            let _log2_min_pcm_luma_coding_block_size_minus3 = r.read_ue()?;
            let _log2_diff_max_min_pcm_luma_coding_block_size = r.read_ue()?;
            let _pcm_loop_filter_disabled_flag = r.read_bit()?;
        }

        Ok(Self {
            sps_id,
            chroma_format_idc,
            bit_depth_luma,
            bit_depth_chroma,
            pic_width_in_luma_samples,
            pic_height_in_luma_samples,
            log2_min_cb_size,
            log2_diff_max_min_cb_size,
            log2_min_tb_size,
            log2_diff_max_min_tb_size,
            max_transform_hierarchy_depth_intra,
            scaling_list_enabled,
            sample_adaptive_offset_enabled,
            pcm_enabled,
        })
    }
}

fn parse_profile_tier_level(r: &mut BitReader, max_sub_layers_minus1: u8) -> HeicResult<()> {
    let _general_profile_space = r.read_bits(2)?;
    let _general_tier_flag = r.read_bit()?;
    let _general_profile_idc = r.read_bits(5)?;
    let _general_profile_compatibility_flags = r.read_bits(32)?;
    let _general_progressive_source_flag = r.read_bit()?;
    let _general_interlaced_source_flag = r.read_bit()?;
    let _general_non_packed_constraint_flag = r.read_bit()?;
    let _general_frame_only_constraint_flag = r.read_bit()?;
    let _reserved44 = r.read_bits(32)?;
    let _reserved12 = r.read_bits(12)?;
    let _general_level_idc = r.read_bits(8)?;

    let mut sub_layer_profile_present_flag = [false; 8];
    let mut sub_layer_level_present_flag = [false; 8];
    for i in 0..max_sub_layers_minus1 as usize {
        sub_layer_profile_present_flag[i] = r.read_bit()? != 0;
        sub_layer_level_present_flag[i] = r.read_bit()? != 0;
    }

    if max_sub_layers_minus1 > 0 {
        for _ in max_sub_layers_minus1..8 {
            let _reserved2 = r.read_bits(2)?;
        }
    }

    for i in 0..max_sub_layers_minus1 as usize {
        if sub_layer_profile_present_flag[i] {
            let _sub_layer_profile_space = r.read_bits(2)?;
            let _sub_layer_tier_flag = r.read_bit()?;
            let _sub_layer_profile_idc = r.read_bits(5)?;
            let _sub_layer_profile_compatibility_flags = r.read_bits(32)?;
            let _sub_layer_progressive_source_flag = r.read_bit()?;
            let _sub_layer_interlaced_source_flag = r.read_bit()?;
            let _sub_layer_non_packed_constraint_flag = r.read_bit()?;
            let _sub_layer_frame_only_constraint_flag = r.read_bit()?;
            let _sub_layer_reserved44 = r.read_bits(32)?;
            let _sub_layer_reserved12 = r.read_bits(12)?;
        }
        if sub_layer_level_present_flag[i] {
            let _sub_layer_level_idc = r.read_bits(8)?;
        }
    }

    Ok(())
}

fn skip_scaling_list_data(r: &mut BitReader) -> HeicResult<()> {
    for size_id in 0..4 {
        let num_matrices = if size_id == 3 { 2 } else { 6 };
        for _matrix_id in 0..num_matrices {
            let scaling_list_pred_mode_flag = r.read_bit()? != 0;
            if !scaling_list_pred_mode_flag {
                let _scaling_list_pred_matrix_id_delta = r.read_ue()?;
            } else {
                let coef_num = (1 << (4 + (size_id << 1))).min(64);
                if size_id > 1 {
                    let _scaling_list_dc_coef_minus8 = r.read_se()?;
                }
                for _ in 0..coef_num {
                    let _scaling_list_delta_coef = r.read_se()?;
                }
            }
        }
    }
    Ok(())
}
