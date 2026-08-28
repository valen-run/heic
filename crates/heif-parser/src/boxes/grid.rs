//! `grid` derived image item configuration descriptor (ISO/IEC 23008-12).

use valen_heic_core::{HeicError, HeicResult, ImageDimensions, Limits};

/// Reconstructed grid image configuration descriptor parsed from a `grid` item's payload data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageGrid {
    /// Number of tile rows in the grid.
    pub rows: u32,
    /// Number of tile columns in the grid.
    pub columns: u32,
    /// Declared full reconstructed output width in pixels.
    pub output_width: u32,
    /// Declared full reconstructed output height in pixels.
    pub output_height: u32,
}

impl ImageGrid {
    /// Total number of tiles required by the grid (`rows * columns`).
    pub const fn tile_count(&self) -> u32 {
        self.rows * self.columns
    }

    /// Reconstructed dimensions.
    pub const fn output_dimensions(&self) -> ImageDimensions {
        ImageDimensions::new(self.output_width, self.output_height)
    }

    /// Parses an `ImageGrid` descriptor from the payload bytes of a `grid` item.
    pub fn parse(payload: &[u8], limits: &Limits) -> HeicResult<Self> {
        if payload.len() < 8 {
            return Err(HeicError::MalformedInput(
                "Insufficient bytes for ImageGrid payload (minimum 8 bytes)".to_string(),
            ));
        }

        let _version = payload[0];
        let flags = payload[1];
        let is_32bit_field = (flags & 1) != 0;

        let rows_minus_one = payload[2] as u32;
        let columns_minus_one = payload[3] as u32;

        let rows = rows_minus_one + 1;
        let columns = columns_minus_one + 1;
        let tile_count = rows * columns;

        limits.check_tile_count(tile_count as usize)?;

        let (output_width, output_height) = if is_32bit_field {
            if payload.len() < 12 {
                return Err(HeicError::MalformedInput(
                    "Insufficient bytes for 32-bit ImageGrid dimensions".to_string(),
                ));
            }
            let w = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
            let h = u32::from_be_bytes([payload[8], payload[9], payload[10], payload[11]]);
            (w, h)
        } else {
            if payload.len() < 8 {
                return Err(HeicError::MalformedInput(
                    "Insufficient bytes for 16-bit ImageGrid dimensions".to_string(),
                ));
            }
            let w = u16::from_be_bytes([payload[4], payload[5]]) as u32;
            let h = u16::from_be_bytes([payload[6], payload[7]]) as u32;
            (w, h)
        };

        let grid = Self {
            rows,
            columns,
            output_width,
            output_height,
        };

        limits.check_dimensions(grid.output_dimensions())?;

        Ok(grid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_16bit_grid() {
        let limits = Limits::none();
        let payload = [
            0x00, // version
            0x00, // flags (16-bit)
            0x01, // rows_minus_one = 1 -> 2 rows
            0x01, // columns_minus_one = 1 -> 2 cols
            0x0f, 0x00, // width = 3840
            0x08, 0x70, // height = 2160
        ];

        let grid = ImageGrid::parse(&payload, &limits).unwrap();
        assert_eq!(grid.rows, 2);
        assert_eq!(grid.columns, 2);
        assert_eq!(grid.tile_count(), 4);
        assert_eq!(grid.output_width, 3840);
        assert_eq!(grid.output_height, 2160);
    }

    #[test]
    fn test_parse_32bit_grid() {
        let limits = Limits::none();
        let payload = [
            0x00, // version
            0x01, // flags (32-bit)
            0x03, // rows_minus_one = 3 -> 4 rows
            0x03, // columns_minus_one = 3 -> 4 cols
            0x00, 0x00, 0x1e, 0x00, // width = 7680
            0x00, 0x00, 0x10, 0xe0, // height = 4320
        ];

        let grid = ImageGrid::parse(&payload, &limits).unwrap();
        assert_eq!(grid.rows, 4);
        assert_eq!(grid.columns, 4);
        assert_eq!(grid.tile_count(), 16);
        assert_eq!(grid.output_width, 7680);
        assert_eq!(grid.output_height, 4320);
    }
}
