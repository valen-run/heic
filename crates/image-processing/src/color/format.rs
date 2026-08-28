//! Pixel format conversion primitives.

use crate::pixels::PixelBuffer;
use valen_heic_core::{HeicResult, Limits, PixelFormat};

/// Converts a pixel buffer to a target [`PixelFormat`].
pub fn convert_pixel_format(
    src: &PixelBuffer,
    target: PixelFormat,
    limits: &Limits,
) -> HeicResult<PixelBuffer> {
    if src.format == target {
        return Ok(src.clone());
    }

    let dims = src.dimensions;
    let mut dst = PixelBuffer::new_with_limits(dims, target, limits)?;

    let src_bpp = src.format.bytes_per_pixel();
    let dst_bpp = target.bytes_per_pixel();

    for y in 0..dims.height {
        let src_row = (y as usize) * src.stride;
        let dst_row = (y as usize) * dst.stride;

        for x in 0..dims.width {
            let src_idx = src_row + (x as usize) * src_bpp;
            let dst_idx = dst_row + (x as usize) * dst_bpp;

            let (r, g, b, a) = match src.format {
                PixelFormat::Rgb8 => (
                    src.data[src_idx],
                    src.data[src_idx + 1],
                    src.data[src_idx + 2],
                    255u8,
                ),
                PixelFormat::Rgba8 => (
                    src.data[src_idx],
                    src.data[src_idx + 1],
                    src.data[src_idx + 2],
                    src.data[src_idx + 3],
                ),
                PixelFormat::Bgr8 => (
                    src.data[src_idx + 2],
                    src.data[src_idx + 1],
                    src.data[src_idx],
                    255u8,
                ),
                PixelFormat::Bgra8 => (
                    src.data[src_idx + 2],
                    src.data[src_idx + 1],
                    src.data[src_idx],
                    src.data[src_idx + 3],
                ),
                _ => (
                    src.data[src_idx],
                    src.data[src_idx],
                    src.data[src_idx],
                    255u8,
                ),
            };

            match target {
                PixelFormat::Rgb8 => {
                    dst.data[dst_idx] = r;
                    dst.data[dst_idx + 1] = g;
                    dst.data[dst_idx + 2] = b;
                }
                PixelFormat::Rgba8 => {
                    dst.data[dst_idx] = r;
                    dst.data[dst_idx + 1] = g;
                    dst.data[dst_idx + 2] = b;
                    dst.data[dst_idx + 3] = a;
                }
                PixelFormat::Bgr8 => {
                    dst.data[dst_idx] = b;
                    dst.data[dst_idx + 1] = g;
                    dst.data[dst_idx + 2] = r;
                }
                PixelFormat::Bgra8 => {
                    dst.data[dst_idx] = b;
                    dst.data[dst_idx + 1] = g;
                    dst.data[dst_idx + 2] = r;
                    dst.data[dst_idx + 3] = a;
                }
                _ => {
                    dst.data[dst_idx] = r;
                    dst.data[dst_idx + 1] = g;
                    dst.data[dst_idx + 2] = b;
                    if dst_bpp >= 4 {
                        dst.data[dst_idx + 3] = a;
                    }
                }
            }
        }
    }

    Ok(dst)
}
