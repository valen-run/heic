//! Auxiliary alpha channel merging and background blending.

use crate::pixels::PixelBuffer;
use valen_heic_core::{HeicError, HeicResult, Limits, PixelFormat};

/// Merges an auxiliary alpha channel buffer into an RGB pixel buffer to produce an RGBA buffer.
pub fn merge_alpha_channel(
    rgb_buf: &PixelBuffer,
    alpha_buf: &PixelBuffer,
    limits: &Limits,
) -> HeicResult<PixelBuffer> {
    if rgb_buf.dimensions != alpha_buf.dimensions {
        return Err(HeicError::InvalidInput(format!(
            "Dimension mismatch between color ({}x{}) and alpha ({}x{}) planes",
            rgb_buf.dimensions.width,
            rgb_buf.dimensions.height,
            alpha_buf.dimensions.width,
            alpha_buf.dimensions.height
        )));
    }

    let dims = rgb_buf.dimensions;
    let mut rgba_buf = PixelBuffer::new_with_limits(dims, PixelFormat::Rgba8, limits)?;

    let rgb_bpp = rgb_buf.format.bytes_per_pixel();
    let alpha_bpp = alpha_buf.format.bytes_per_pixel();

    for y in 0..dims.height {
        let rgb_row = (y as usize) * rgb_buf.stride;
        let alpha_row = (y as usize) * alpha_buf.stride;
        let rgba_row = (y as usize) * rgba_buf.stride;

        for x in 0..dims.width {
            let rgb_idx = rgb_row + (x as usize) * rgb_bpp;
            let alpha_idx = alpha_row + (x as usize) * alpha_bpp;
            let rgba_idx = rgba_row + (x as usize) * 4;

            let (r, g, b) = match rgb_buf.format {
                PixelFormat::Rgb8 | PixelFormat::Rgba8 => (
                    rgb_buf.data[rgb_idx],
                    rgb_buf.data[rgb_idx + 1],
                    rgb_buf.data[rgb_idx + 2],
                ),
                PixelFormat::Bgr8 | PixelFormat::Bgra8 => (
                    rgb_buf.data[rgb_idx + 2],
                    rgb_buf.data[rgb_idx + 1],
                    rgb_buf.data[rgb_idx],
                ),
                _ => (
                    rgb_buf.data[rgb_idx],
                    rgb_buf.data[rgb_idx
                        .saturating_add(1)
                        .min(rgb_buf.data.len().saturating_sub(1))],
                    rgb_buf.data[rgb_idx
                        .saturating_add(2)
                        .min(rgb_buf.data.len().saturating_sub(1))],
                ),
            };

            let a = if alpha_bpp >= 4 {
                alpha_buf.data[alpha_idx + 3]
            } else {
                alpha_buf.data[alpha_idx]
            };

            rgba_buf.data[rgba_idx] = r;
            rgba_buf.data[rgba_idx + 1] = g;
            rgba_buf.data[rgba_idx + 2] = b;
            rgba_buf.data[rgba_idx + 3] = a;
        }
    }

    Ok(rgba_buf)
}

/// Flattens a semi-transparent RGBA buffer onto a solid background color.
pub fn flatten_alpha(
    rgba_buf: &PixelBuffer,
    bg_color: [u8; 3],
    limits: &Limits,
) -> HeicResult<PixelBuffer> {
    let dims = rgba_buf.dimensions;
    let mut rgb_buf = PixelBuffer::new_with_limits(dims, PixelFormat::Rgb8, limits)?;
    let src_bpp = rgba_buf.format.bytes_per_pixel();

    for y in 0..dims.height {
        let src_row = (y as usize) * rgba_buf.stride;
        let dst_row = (y as usize) * rgb_buf.stride;

        for x in 0..dims.width {
            let src_idx = src_row + (x as usize) * src_bpp;
            let dst_idx = dst_row + (x as usize) * 3;

            let (r, g, b, a) = match rgba_buf.format {
                PixelFormat::Rgba8 => (
                    rgba_buf.data[src_idx] as u32,
                    rgba_buf.data[src_idx + 1] as u32,
                    rgba_buf.data[src_idx + 2] as u32,
                    rgba_buf.data[src_idx + 3] as u32,
                ),
                PixelFormat::Bgra8 => (
                    rgba_buf.data[src_idx + 2] as u32,
                    rgba_buf.data[src_idx + 1] as u32,
                    rgba_buf.data[src_idx] as u32,
                    rgba_buf.data[src_idx + 3] as u32,
                ),
                _ => (
                    rgba_buf.data[src_idx] as u32,
                    rgba_buf.data[src_idx + 1] as u32,
                    rgba_buf.data[src_idx + 2] as u32,
                    255u32,
                ),
            };

            let out_r = ((r * a + (bg_color[0] as u32) * (255 - a) + 127) / 255) as u8;
            let out_g = ((g * a + (bg_color[1] as u32) * (255 - a) + 127) / 255) as u8;
            let out_b = ((b * a + (bg_color[2] as u32) * (255 - a) + 127) / 255) as u8;

            rgb_buf.data[dst_idx] = out_r;
            rgb_buf.data[dst_idx + 1] = out_g;
            rgb_buf.data[dst_idx + 2] = out_b;
        }
    }

    Ok(rgb_buf)
}
