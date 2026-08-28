//! Coding Tree Unit (CTU) quadtree traversal and Intra CU reconstruction.

use crate::cabac::{CabacContexts, CabacEngine};
use crate::intra_pred::{predict_intra, IntraReferences};
use crate::nal::{Pps, Sps};
use crate::reconstruct::frame::PlanarFrame;
use crate::transform::{dequantize_block, inverse_transform};
use valen_heic_core::HeicResult;

/// Recursively traverses and decodes the Coding Unit (CU) quadtree within a CTU.
#[allow(clippy::too_many_arguments)]
pub fn decode_cu_quadtree(
    cabac: &mut CabacEngine,
    contexts: &mut CabacContexts,
    frame: &mut PlanarFrame,
    sps: &Sps,
    _pps: &Pps,
    x0: usize,
    y0: usize,
    cu_size: usize,
    depth: usize,
    qp: i32,
    bit_depth: u8,
    max_val: u16,
) -> HeicResult<()> {
    if x0 >= frame.width || y0 >= frame.height {
        return Ok(());
    }

    let min_cb_size = sps.min_cb_size() as usize;
    let can_split = cu_size > min_cb_size;

    let split_flag = if can_split {
        if x0 + cu_size > frame.width || y0 + cu_size > frame.height {
            1 // Forced boundary split
        } else {
            let ctx_idx = (depth).min(2);
            cabac.decode_bin(&mut contexts.split_cu_flag[ctx_idx])?
        }
    } else {
        0
    };

    if split_flag != 0 {
        let half = cu_size / 2;
        decode_cu_quadtree(
            cabac,
            contexts,
            frame,
            sps,
            _pps,
            x0,
            y0,
            half,
            depth + 1,
            qp,
            bit_depth,
            max_val,
        )?;
        decode_cu_quadtree(
            cabac,
            contexts,
            frame,
            sps,
            _pps,
            x0 + half,
            y0,
            half,
            depth + 1,
            qp,
            bit_depth,
            max_val,
        )?;
        decode_cu_quadtree(
            cabac,
            contexts,
            frame,
            sps,
            _pps,
            x0,
            y0 + half,
            half,
            depth + 1,
            qp,
            bit_depth,
            max_val,
        )?;
        decode_cu_quadtree(
            cabac,
            contexts,
            frame,
            sps,
            _pps,
            x0 + half,
            y0 + half,
            half,
            depth + 1,
            qp,
            bit_depth,
            max_val,
        )?;
    } else {
        // Decode Coding Unit (Intra)
        decode_intra_cu(
            cabac, contexts, frame, x0, y0, cu_size, qp, bit_depth, max_val,
        )?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn decode_intra_cu(
    cabac: &mut CabacEngine,
    contexts: &mut CabacContexts,
    frame: &mut PlanarFrame,
    x0: usize,
    y0: usize,
    cu_size: usize,
    qp: i32,
    bit_depth: u8,
    max_val: u16,
) -> HeicResult<()> {
    // 1. Decode Intra Luma Mode
    let prev_intra_luma_pred_flag = cabac.decode_bin(&mut contexts.prev_intra_luma_pred_flag)? != 0;
    let luma_mode = if prev_intra_luma_pred_flag {
        let mpm_idx = cabac.decode_bins_bypass(1)?;
        match mpm_idx {
            0 => 0,  // Planar
            1 => 1,  // DC
            _ => 26, // Vertical
        }
    } else {
        let rem_mode = cabac.decode_bins_bypass(5)? as u8;
        rem_mode.saturating_add(2)
    };

    // 2. Extract neighboring references
    let block_size = cu_size.min(32);
    let mut refs = IntraReferences::new(block_size, 1 << (bit_depth - 1));

    // Fill Top reference samples
    if y0 > 0 {
        refs.top[0] = if x0 > 0 {
            frame.y[(y0 - 1) * frame.y_stride + x0 - 1]
        } else {
            frame.y[(y0 - 1) * frame.y_stride]
        };
        for i in 0..2 * block_size {
            let px = (x0 + i).min(frame.width.saturating_sub(1));
            refs.top[i + 1] = frame.y[(y0 - 1) * frame.y_stride + px];
        }
    }

    // Fill Left reference samples
    if x0 > 0 {
        refs.left[0] = refs.top[0];
        for i in 0..2 * block_size {
            let py = (y0 + i).min(frame.height.saturating_sub(1));
            refs.left[i + 1] = frame.y[py * frame.y_stride + x0 - 1];
        }
    }

    let filtered_refs = refs.filter_references(luma_mode, true, bit_depth);

    // 3. Intra Prediction
    let mut pred_block = vec![0u16; block_size * block_size];
    predict_intra(luma_mode, &filtered_refs, &mut pred_block, block_size, true);

    // 4. Decode Residual coefficients
    let cbf_luma = cabac.decode_bin(&mut contexts.cbf_luma[0])? != 0;
    let mut residual = vec![0i32; block_size * block_size];

    if cbf_luma {
        let mut dequant = vec![0i32; block_size * block_size];
        // Read DC/low-frequency residual coefficients
        let num_coeffs = block_size.min(4) * block_size.min(4);
        for item in dequant.iter_mut().take(num_coeffs) {
            let sig = cabac.decode_bin(&mut contexts.significant_coeff_flag[0])?;
            if sig != 0 {
                let sign = cabac.decode_bypass_bin()?;
                let level = 1 + cabac.decode_coeff_abs_level_remaining(0)? as i32;
                *item = if sign != 0 { -level } else { level };
            }
        }

        dequantize_block(&dequant, &mut residual, block_size, qp, bit_depth, false);
        let mut idct_out = vec![0i32; block_size * block_size];
        inverse_transform(
            &residual,
            &mut idct_out,
            block_size,
            true,
            luma_mode,
            bit_depth,
            false,
        );
        residual = idct_out;
    }

    // 5. Sample Reconstruction: Sample = clip(Pred + Res, 0, max_val)
    for y in 0..block_size {
        if y0 + y >= frame.height {
            break;
        }
        for x in 0..block_size {
            if x0 + x >= frame.width {
                break;
            }
            let pred_val = pred_block[y * block_size + x] as i32;
            let res_val = residual[y * block_size + x];
            let recon = (pred_val + res_val).clamp(0, max_val as i32) as u16;
            frame.y[(y0 + y) * frame.y_stride + x0 + x] = recon;
        }
    }

    Ok(())
}
