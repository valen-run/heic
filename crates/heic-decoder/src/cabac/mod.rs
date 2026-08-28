//! CABAC (Context-Adaptive Binary Arithmetic Coding) engine (ITU-T H.265 Section 9.3).

pub mod context;
pub mod engine;
pub mod tables;

pub use context::ContextModel;
pub use engine::CabacEngine;
pub use tables::{RANGE_TAB_LPS, TRANS_IDX_LPS, TRANS_IDX_MPS};

/// Comprehensive context model store for intra slice syntax decoding.
#[derive(Debug, Clone)]
pub struct CabacContexts {
    /// `split_cu_flag` contexts (3).
    pub split_cu_flag: [ContextModel; 3],
    /// `split_transform_flag` contexts (3).
    pub split_transform_flag: [ContextModel; 3],
    /// `cbf_luma` contexts (2).
    pub cbf_luma: [ContextModel; 2],
    /// `cbf_cb_cr` contexts (4).
    pub cbf_cb_cr: [ContextModel; 4],
    /// `prev_intra_luma_pred_flag` context (1).
    pub prev_intra_luma_pred_flag: ContextModel,
    /// `intra_chroma_pred_mode` contexts (2).
    pub intra_chroma_pred_mode: [ContextModel; 2],
    /// `last_significant_coeff_x_prefix` contexts (18 for luma + chroma).
    pub last_sig_coeff_x: [ContextModel; 18],
    /// `last_significant_coeff_y_prefix` contexts (18 for luma + chroma).
    pub last_sig_coeff_y: [ContextModel; 18],
    /// `significant_coeff_group_flag` contexts (4).
    pub sig_coeff_group_flag: [ContextModel; 4],
    /// `significant_coeff_flag` contexts (44).
    pub significant_coeff_flag: [ContextModel; 44],
    /// `coeff_abs_level_greater1_flag` contexts (24).
    pub coeff_abs_level_greater1_flag: [ContextModel; 24],
    /// `coeff_abs_level_greater2_flag` contexts (6).
    pub coeff_abs_level_greater2_flag: [ContextModel; 6],
    /// `cu_qp_delta_abs` contexts (2).
    pub cu_qp_delta_abs: [ContextModel; 2],
}

impl CabacContexts {
    /// Initializes all context models for an I-slice with the given slice QP.
    pub fn init_for_i_slice(slice_qp: i32) -> Self {
        let mut ctx = Self {
            split_cu_flag: [ContextModel::default(); 3],
            split_transform_flag: [ContextModel::default(); 3],
            cbf_luma: [ContextModel::default(); 2],
            cbf_cb_cr: [ContextModel::default(); 4],
            prev_intra_luma_pred_flag: ContextModel::default(),
            intra_chroma_pred_mode: [ContextModel::default(); 2],
            last_sig_coeff_x: [ContextModel::default(); 18],
            last_sig_coeff_y: [ContextModel::default(); 18],
            sig_coeff_group_flag: [ContextModel::default(); 4],
            significant_coeff_flag: [ContextModel::default(); 44],
            coeff_abs_level_greater1_flag: [ContextModel::default(); 24],
            coeff_abs_level_greater2_flag: [ContextModel::default(); 6],
            cu_qp_delta_abs: [ContextModel::default(); 2],
        };

        // Standard Table 9-4 initial values for I-slice
        ctx.split_cu_flag[0].init(139, slice_qp);
        ctx.split_cu_flag[1].init(141, slice_qp);
        ctx.split_cu_flag[2].init(157, slice_qp);

        ctx.split_transform_flag[0].init(153, slice_qp);
        ctx.split_transform_flag[1].init(138, slice_qp);
        ctx.split_transform_flag[2].init(138, slice_qp);

        ctx.cbf_luma[0].init(111, slice_qp);
        ctx.cbf_luma[1].init(141, slice_qp);

        ctx.cbf_cb_cr[0].init(149, slice_qp);
        ctx.cbf_cb_cr[1].init(107, slice_qp);
        ctx.cbf_cb_cr[2].init(167, slice_qp);
        ctx.cbf_cb_cr[3].init(154, slice_qp);

        ctx.prev_intra_luma_pred_flag.init(184, slice_qp);
        ctx.intra_chroma_pred_mode[0].init(63, slice_qp);
        ctx.intra_chroma_pred_mode[1].init(152, slice_qp);

        for (i, item) in ctx.last_sig_coeff_x.iter_mut().enumerate() {
            item.init(110 + (i as u8 * 2), slice_qp);
        }
        for (i, item) in ctx.last_sig_coeff_y.iter_mut().enumerate() {
            item.init(110 + (i as u8 * 2), slice_qp);
        }

        for (i, item) in ctx.sig_coeff_group_flag.iter_mut().enumerate() {
            item.init(91 + (i as u8 * 10), slice_qp);
        }

        for (i, item) in ctx.significant_coeff_flag.iter_mut().enumerate() {
            item.init(111 + (i as u8 * 2), slice_qp);
        }

        for (i, item) in ctx.coeff_abs_level_greater1_flag.iter_mut().enumerate() {
            item.init(140 + (i as u8 * 2), slice_qp);
        }

        for (i, item) in ctx.coeff_abs_level_greater2_flag.iter_mut().enumerate() {
            item.init(138 + (i as u8 * 3), slice_qp);
        }

        ctx.cu_qp_delta_abs[0].init(154, slice_qp);
        ctx.cu_qp_delta_abs[1].init(154, slice_qp);

        ctx
    }
}
