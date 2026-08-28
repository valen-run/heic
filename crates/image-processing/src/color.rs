//! Color spaces, auxiliary alpha channel compositing, background blending, and pixel format conversion.

use crate::pixels::PixelBuffer;
use valen_heic_core::{ColorSpace, HeicError, HeicResult, Limits, PixelFormat};

/// Color profile and color transfer description.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ColorProfileInfo {
    /// Identified color space.
    pub space: ColorSpace,
    /// Raw ICC profile data if embedded in the container.
    pub raw_icc: Option<Vec<u8>>,
}

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

            // Extract RGB components according to format
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

            // Alpha sample (first byte of alpha buffer or 4th byte if RGBA)
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

/// Flattens a semi-transparent RGBA buffer onto a solid background color (e.g. `[255, 255, 255]` for white).
///
/// Blending formula with rounding:
/// `C_out = (C_src * alpha + C_bg * (255 - alpha) + 127) / 255`
///
/// Returns an opaque [`PixelFormat::Rgb8`] pixel buffer.
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

#[cfg(test)]
mod tests {
    use super::*;
    use valen_heic_core::ImageDimensions;

    #[test]
    fn test_merge_alpha_channel() {
        let limits = Limits::none();
        let dims = ImageDimensions::new(2, 2);

        let mut rgb = PixelBuffer::new(dims, PixelFormat::Rgb8);
        rgb.fill(&[100, 150, 200]).unwrap();

        let mut alpha = PixelBuffer::new(dims, PixelFormat::Rgb8);
        alpha.fill(&[128, 128, 128]).unwrap();

        let rgba = merge_alpha_channel(&rgb, &alpha, &limits).unwrap();
        assert_eq!(rgba.format, PixelFormat::Rgba8);
        assert_eq!(rgba.get_pixel(0, 0), Some(&[100, 150, 200, 128][..]));
    }

    #[test]
    fn test_flatten_alpha_onto_white() {
        let limits = Limits::none();
        let dims = ImageDimensions::new(1, 1);

        let mut rgba = PixelBuffer::new(dims, PixelFormat::Rgba8);
        // 50% transparent red
        rgba.set_pixel(0, 0, &[255, 0, 0, 128]).unwrap();

        let rgb = flatten_alpha(&rgba, [255, 255, 255], &limits).unwrap();
        assert_eq!(rgb.format, PixelFormat::Rgb8);
        let px = rgb.get_pixel(0, 0).unwrap();
        // Red blended over white should be around (255, 127, 127)
        assert_eq!(px[0], 255);
        assert!((px[1] as i32 - 127).abs() <= 1);
        assert!((px[2] as i32 - 127).abs() <= 1);
    }

    #[test]
    fn test_convert_pixel_format_rgb_to_bgra() {
        let limits = Limits::none();
        let dims = ImageDimensions::new(1, 1);

        let mut rgb = PixelBuffer::new(dims, PixelFormat::Rgb8);
        rgb.set_pixel(0, 0, &[10, 20, 30]).unwrap();

        let bgra = convert_pixel_format(&rgb, PixelFormat::Bgra8, &limits).unwrap();
        assert_eq!(bgra.format, PixelFormat::Bgra8);
        assert_eq!(bgra.get_pixel(0, 0), Some(&[30, 20, 10, 255][..]));
    }
}
