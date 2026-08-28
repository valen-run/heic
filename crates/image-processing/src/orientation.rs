//! EXIF orientation transformations (rotations and reflections).

use crate::pixels::PixelBuffer;
pub use valen_heic_core::Orientation;
use valen_heic_core::{HeicResult, ImageDimensions, Limits};

/// Alias for [`Orientation`] for backward compatibility with EXIF nomenclature.
pub type ExifOrientation = Orientation;

/// Applies an EXIF orientation transformation (1..=8) to the pixel buffer.
///
/// Geometric mappings:
/// - `1 (Normal)`: 0° rotation (identity)
/// - `2 (MirrorHorizontal)`: Flip horizontally across vertical axis `(W - 1 - x, y)`
/// - `3 (Rotate180)`: 180° rotation `(W - 1 - x, H - 1 - y)`
/// - `4 (MirrorVertical)`: Flip vertically across horizontal axis `(x, H - 1 - y)`
/// - `5 (MirrorHorizontalRotate270)`: Transpose along main diagonal `(y, x)`
/// - `6 (Rotate90)`: 90° clockwise rotation `(H - 1 - y, x)`
/// - `7 (MirrorHorizontalRotate90)`: Transverse reflection along anti-diagonal `(H - 1 - y, W - 1 - x)`
/// - `8 (Rotate270)`: 270° clockwise (90° counter-clockwise) rotation `(y, W - 1 - x)`
pub fn apply_orientation(
    src: &PixelBuffer,
    orientation: Orientation,
    limits: &Limits,
) -> HeicResult<PixelBuffer> {
    if orientation == Orientation::Normal {
        return Ok(src.clone());
    }

    let src_w = src.dimensions.width;
    let src_h = src.dimensions.height;
    let bpp = src.format.bytes_per_pixel();

    let (dst_w, dst_h) = if orientation.swaps_dimensions() {
        (src_h, src_w)
    } else {
        (src_w, src_h)
    };

    let dst_dims = ImageDimensions::new(dst_w, dst_h);
    let mut dst = PixelBuffer::new_with_limits(dst_dims, src.format, limits)?;

    for src_y in 0..src_h {
        let src_row_start = (src_y as usize) * src.stride;

        for src_x in 0..src_w {
            let (dst_x, dst_y) = match orientation {
                Orientation::Normal => (src_x, src_y),
                Orientation::MirrorHorizontal => (src_w - 1 - src_x, src_y),
                Orientation::Rotate180 => (src_w - 1 - src_x, src_h - 1 - src_y),
                Orientation::MirrorVertical => (src_x, src_h - 1 - src_y),
                Orientation::MirrorHorizontalRotate270 => (src_y, src_x),
                Orientation::Rotate90 => (src_h - 1 - src_y, src_x),
                Orientation::MirrorHorizontalRotate90 => (src_h - 1 - src_y, src_w - 1 - src_x),
                Orientation::Rotate270 => (src_y, src_w - 1 - src_x),
            };

            let src_idx = src_row_start + (src_x as usize) * bpp;
            let dst_idx = (dst_y as usize) * dst.stride + (dst_x as usize) * bpp;

            dst.data[dst_idx..dst_idx + bpp].copy_from_slice(&src.data[src_idx..src_idx + bpp]);
        }
    }

    Ok(dst)
}

#[cfg(test)]
mod tests {
    use super::*;
    use valen_heic_core::PixelFormat;

    #[test]
    fn test_orientation_transforms() {
        let limits = Limits::none();
        // 2x3 test image
        // [ [1, 2],
        //   [3, 4],
        //   [5, 6] ]
        let mut src = PixelBuffer::new(ImageDimensions::new(2, 3), PixelFormat::Rgb8);
        src.set_pixel(0, 0, &[1, 1, 1]).unwrap();
        src.set_pixel(1, 0, &[2, 2, 2]).unwrap();
        src.set_pixel(0, 1, &[3, 3, 3]).unwrap();
        src.set_pixel(1, 1, &[4, 4, 4]).unwrap();
        src.set_pixel(0, 2, &[5, 5, 5]).unwrap();
        src.set_pixel(1, 2, &[6, 6, 6]).unwrap();

        // 1. Normal
        let normal = apply_orientation(&src, Orientation::Normal, &limits).unwrap();
        assert_eq!(normal.dimensions, ImageDimensions::new(2, 3));
        assert_eq!(normal.get_pixel(0, 0), Some(&[1, 1, 1][..]));

        // 2. MirrorHorizontal (FlipH)
        let flip_h = apply_orientation(&src, Orientation::MirrorHorizontal, &limits).unwrap();
        assert_eq!(flip_h.dimensions, ImageDimensions::new(2, 3));
        assert_eq!(flip_h.get_pixel(0, 0), Some(&[2, 2, 2][..]));
        assert_eq!(flip_h.get_pixel(1, 0), Some(&[1, 1, 1][..]));

        // 3. Rotate180
        let rot180 = apply_orientation(&src, Orientation::Rotate180, &limits).unwrap();
        assert_eq!(rot180.dimensions, ImageDimensions::new(2, 3));
        assert_eq!(rot180.get_pixel(0, 0), Some(&[6, 6, 6][..]));
        assert_eq!(rot180.get_pixel(1, 2), Some(&[1, 1, 1][..]));

        // 4. MirrorVertical (FlipV)
        let flip_v = apply_orientation(&src, Orientation::MirrorVertical, &limits).unwrap();
        assert_eq!(flip_v.dimensions, ImageDimensions::new(2, 3));
        assert_eq!(flip_v.get_pixel(0, 0), Some(&[5, 5, 5][..]));
        assert_eq!(flip_v.get_pixel(1, 0), Some(&[6, 6, 6][..]));

        // 5. MirrorHorizontalRotate270 (Transpose)
        let transpose =
            apply_orientation(&src, Orientation::MirrorHorizontalRotate270, &limits).unwrap();
        assert_eq!(transpose.dimensions, ImageDimensions::new(3, 2));
        assert_eq!(transpose.get_pixel(0, 0), Some(&[1, 1, 1][..]));
        assert_eq!(transpose.get_pixel(2, 0), Some(&[5, 5, 5][..]));

        // 6. Rotate90
        let rot90 = apply_orientation(&src, Orientation::Rotate90, &limits).unwrap();
        assert_eq!(rot90.dimensions, ImageDimensions::new(3, 2));
        assert_eq!(rot90.get_pixel(0, 0), Some(&[5, 5, 5][..]));
        assert_eq!(rot90.get_pixel(0, 1), Some(&[6, 6, 6][..]));
        assert_eq!(rot90.get_pixel(2, 0), Some(&[1, 1, 1][..]));

        // 7. MirrorHorizontalRotate90 (Transverse)
        let transverse =
            apply_orientation(&src, Orientation::MirrorHorizontalRotate90, &limits).unwrap();
        assert_eq!(transverse.dimensions, ImageDimensions::new(3, 2));
        assert_eq!(transverse.get_pixel(0, 0), Some(&[6, 6, 6][..]));

        // 8. Rotate270
        let rot270 = apply_orientation(&src, Orientation::Rotate270, &limits).unwrap();
        assert_eq!(rot270.dimensions, ImageDimensions::new(3, 2));
        assert_eq!(rot270.get_pixel(0, 0), Some(&[2, 2, 2][..]));
        assert_eq!(rot270.get_pixel(2, 1), Some(&[5, 5, 5][..]));
    }
}
