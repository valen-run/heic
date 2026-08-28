//! ISO/IEC 23008-12 Multi-tile image grid assembly.

use crate::pixels::PixelBuffer;
use valen_heic_core::{HeicError, HeicResult, ImageDimensions, Limits};

/// Assembles a multi-tile image grid into a single contiguous [`PixelBuffer`].
///
/// Tiles are ordered in row-major order: `tile[0] = (r=0, c=0)`, `tile[1] = (r=0, c=1)`, etc.
/// If the combined tiles exceed `output_width` or `output_height`, the excess edges are cropped.
pub fn assemble_grid(
    tiles: &[PixelBuffer],
    rows: u32,
    columns: u32,
    output_width: u32,
    output_height: u32,
    limits: &Limits,
) -> HeicResult<PixelBuffer> {
    if rows == 0 || columns == 0 {
        return Err(HeicError::InvalidContainer(
            "Grid rows and columns must be non-zero".to_string(),
        ));
    }

    let expected_count = (rows as usize).saturating_mul(columns as usize);
    if tiles.len() != expected_count {
        return Err(HeicError::MalformedInput(format!(
            "Grid tile count mismatch: expected {} ({}x{}), got {}",
            expected_count,
            rows,
            columns,
            tiles.len()
        )));
    }

    limits.check_tile_count(tiles.len())?;

    let first_tile = &tiles[0];
    let tile_w = first_tile.dimensions.width;
    let tile_h = first_tile.dimensions.height;
    let format = first_tile.format;

    // Validate tile uniformity
    for (i, tile) in tiles.iter().enumerate() {
        if tile.format != format {
            return Err(HeicError::InvalidContainer(format!(
                "Tile {} format {:?} does not match grid format {:?}",
                i, tile.format, format
            )));
        }
        if tile.dimensions.width != tile_w || tile.dimensions.height != tile_h {
            return Err(HeicError::InvalidContainer(format!(
                "Tile {} dimensions {}x{} do not match first tile {}x{}",
                i, tile.dimensions.width, tile.dimensions.height, tile_w, tile_h
            )));
        }
    }

    let out_dims = ImageDimensions::new(output_width, output_height);
    let mut canvas = PixelBuffer::new_with_limits(out_dims, format, limits)?;

    // Stitch tiles into the canvas
    for r in 0..rows {
        for c in 0..columns {
            let tile_idx = (r * columns + c) as usize;
            let tile = &tiles[tile_idx];

            let dst_x = c * tile_w;
            let dst_y = r * tile_h;

            if dst_x < output_width && dst_y < output_height {
                let copy_w = tile_w.min(output_width.saturating_sub(dst_x));
                let copy_h = tile_h.min(output_height.saturating_sub(dst_y));

                canvas.blit(dst_x, dst_y, tile, 0, 0, copy_w, copy_h)?;
            }
        }
    }

    Ok(canvas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use valen_heic_core::PixelFormat;

    #[test]
    fn test_assemble_grid_2x2() {
        let limits = Limits::none();
        let tile_dims = ImageDimensions::new(4, 4);

        let mut t0 = PixelBuffer::new(tile_dims, PixelFormat::Rgb8);
        t0.fill(&[10, 0, 0]).unwrap();
        let mut t1 = PixelBuffer::new(tile_dims, PixelFormat::Rgb8);
        t1.fill(&[0, 20, 0]).unwrap();
        let mut t2 = PixelBuffer::new(tile_dims, PixelFormat::Rgb8);
        t2.fill(&[0, 0, 30]).unwrap();
        let mut t3 = PixelBuffer::new(tile_dims, PixelFormat::Rgb8);
        t3.fill(&[40, 40, 40]).unwrap();

        let tiles = vec![t0, t1, t2, t3];
        let canvas =
            assemble_grid(&tiles, 2, 2, 8, 8, &limits).expect("Grid assemble should succeed");

        assert_eq!(canvas.dimensions.width, 8);
        assert_eq!(canvas.dimensions.height, 8);
        assert_eq!(canvas.get_pixel(0, 0), Some(&[10, 0, 0][..]));
        assert_eq!(canvas.get_pixel(4, 0), Some(&[0, 20, 0][..]));
        assert_eq!(canvas.get_pixel(0, 4), Some(&[0, 0, 30][..]));
        assert_eq!(canvas.get_pixel(4, 4), Some(&[40, 40, 40][..]));
    }

    #[test]
    fn test_assemble_grid_with_edge_cropping() {
        let limits = Limits::none();
        let tile_dims = ImageDimensions::new(4, 4);

        let mut t0 = PixelBuffer::new(tile_dims, PixelFormat::Rgb8);
        t0.fill(&[10, 0, 0]).unwrap();
        let mut t1 = PixelBuffer::new(tile_dims, PixelFormat::Rgb8);
        t1.fill(&[0, 20, 0]).unwrap();
        let mut t2 = PixelBuffer::new(tile_dims, PixelFormat::Rgb8);
        t2.fill(&[0, 0, 30]).unwrap();
        let mut t3 = PixelBuffer::new(tile_dims, PixelFormat::Rgb8);
        t3.fill(&[40, 40, 40]).unwrap();

        let tiles = vec![t0, t1, t2, t3];
        // Crop from 8x8 to 7x6
        let canvas =
            assemble_grid(&tiles, 2, 2, 7, 6, &limits).expect("Grid assemble should succeed");

        assert_eq!(canvas.dimensions.width, 7);
        assert_eq!(canvas.dimensions.height, 6);
        assert_eq!(canvas.get_pixel(6, 0), Some(&[0, 20, 0][..]));
        assert_eq!(canvas.get_pixel(0, 5), Some(&[0, 0, 30][..]));
        assert_eq!(canvas.get_pixel(6, 5), Some(&[40, 40, 40][..]));
    }

    #[test]
    fn test_assemble_grid_mismatched_tile_count() {
        let limits = Limits::none();
        let t0 = PixelBuffer::new(ImageDimensions::new(4, 4), PixelFormat::Rgb8);
        let res = assemble_grid(&[t0], 2, 2, 8, 8, &limits);
        assert!(res.is_err());
    }
}
