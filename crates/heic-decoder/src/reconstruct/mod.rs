//! Planar frame buffer management, CTU quadtree reconstruction, and YUV-to-RGB conversion.

pub mod ctu;
pub mod frame;

use crate::cabac::{CabacContexts, CabacEngine};
use crate::loop_filter::{apply_sao_edge_offset, deblock_luma_edge, EdgeType};
use crate::nal::{NalUnit, NalUnitType, Pps, SliceHeader, SliceType, Sps};
use crate::reconstruct::ctu::decode_cu_quadtree;
pub use frame::PlanarFrame;
use valen_heic_core::{HeicError, HeicResult, Limits};

/// Decodes an Annex-B HEVC intra bitstream into a reconstructed [`PlanarFrame`].
pub fn decode_intra_bitstream(annex_b: &[u8], limits: &Limits) -> HeicResult<PlanarFrame> {
    let nals = NalUnit::parse_annex_b(annex_b)?;
    if nals.is_empty() {
        return Err(HeicError::MalformedInput(
            "No NAL units found in bitstream".into(),
        ));
    }

    let mut sps: Option<Sps> = None;
    let mut pps: Option<Pps> = None;
    let mut slice_nals = Vec::new();

    for nal in &nals {
        match nal.unit_type {
            NalUnitType::SpsNut => {
                sps = Some(Sps::parse(&nal.rbsp_data, limits)?);
            }
            NalUnitType::PpsNut => {
                pps = Some(Pps::parse(&nal.rbsp_data)?);
            }
            u if u.is_slice() => {
                slice_nals.push(nal);
            }
            _ => {}
        }
    }

    let sps = sps
        .ok_or_else(|| HeicError::InvalidContainer("Missing SPS in HEVC bitstream".to_string()))?;
    let pps = pps
        .ok_or_else(|| HeicError::InvalidContainer("Missing PPS in HEVC bitstream".to_string()))?;

    if slice_nals.is_empty() {
        return Err(HeicError::MalformedInput(
            "No slice NAL units found in bitstream".into(),
        ));
    }

    let width = sps.pic_width_in_luma_samples as usize;
    let height = sps.pic_height_in_luma_samples as usize;
    let mut frame = PlanarFrame::new(width, height, sps.bit_depth_luma, limits)?;

    // Decode primary slice
    let primary_slice_nal = slice_nals[0];
    let (slice_header, byte_offset, _) = SliceHeader::parse(
        &primary_slice_nal.rbsp_data,
        primary_slice_nal.unit_type,
        &sps,
        &pps,
    )?;

    if slice_header.slice_type != SliceType::I {
        return Err(HeicError::UnsupportedFeature(
            "Inter-frame (P/B slice) decoding is not supported in still-image decoder".into(),
        ));
    }

    let mut cabac = CabacEngine::init(&primary_slice_nal.rbsp_data, byte_offset)?;
    let mut contexts = CabacContexts::init_for_i_slice(slice_header.slice_qp);

    let ctu_size = sps.ctu_size() as usize;
    let ctu_width = width.div_ceil(ctu_size);
    let ctu_height = height.div_ceil(ctu_size);

    let qp = slice_header.slice_qp;
    let bit_depth = sps.bit_depth_luma;
    let max_val = (1 << bit_depth) - 1;

    // Loop over CTUs in raster scan order
    for ctu_y in 0..ctu_height {
        for ctu_x in 0..ctu_width {
            let x0 = ctu_x * ctu_size;
            let y0 = ctu_y * ctu_size;

            decode_cu_quadtree(
                &mut cabac,
                &mut contexts,
                &mut frame,
                &sps,
                &pps,
                x0,
                y0,
                ctu_size,
                0,
                qp,
                bit_depth,
                max_val,
            )?;
        }
    }

    // In-loop deblocking filter
    if !slice_header.slice_deblocking_filter_disabled {
        for y in (8..height).step_by(8) {
            for x in (0..width).step_by(8) {
                deblock_luma_edge(
                    &mut frame.y,
                    frame.y_stride,
                    x,
                    y,
                    EdgeType::Horizontal,
                    qp,
                    slice_header.slice_beta_offset_div2 * 2,
                    slice_header.slice_tc_offset_div2 * 2,
                    bit_depth,
                );
            }
        }
        for x in (8..width).step_by(8) {
            for y in (0..height).step_by(8) {
                deblock_luma_edge(
                    &mut frame.y,
                    frame.y_stride,
                    x,
                    y,
                    EdgeType::Vertical,
                    qp,
                    slice_header.slice_beta_offset_div2 * 2,
                    slice_header.slice_tc_offset_div2 * 2,
                    bit_depth,
                );
            }
        }
    }

    // Sample Adaptive Offset (SAO)
    if slice_header.slice_sao_luma_flag {
        let offsets = [1, 1, 1, 1];
        apply_sao_edge_offset(
            &mut frame.y,
            frame.y_stride,
            width,
            height,
            0,
            &offsets,
            bit_depth,
        );
    }

    Ok(frame)
}
