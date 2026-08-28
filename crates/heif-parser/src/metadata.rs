//! Extracted HEIF metadata models and container info.

use valen_heic_core::{ColorSpace, ImageDimensions};

/// High-level container and image metadata extracted from HEIF boxes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerMetadata {
    /// Major brand identifier from `ftyp` (e.g. `heic`, `mif1`).
    pub major_brand: [u8; 4],
    /// Compatible brands declared in `ftyp`.
    pub compatible_brands: Vec<[u8; 4]>,
    /// Primary image dimensions.
    pub dimensions: ImageDimensions,
    /// Color space or profile identifier.
    pub color_space: ColorSpace,
    /// EXIF orientation value (1 through 8) if present.
    pub orientation: Option<u8>,
    /// Primary item ID from `pitm`.
    pub primary_item_id: Option<u32>,
    /// Total number of image items.
    pub image_count: usize,
    /// Whether the primary image is a reconstructed grid (`grid` derived image).
    pub is_grid: bool,
    /// Grid rows if this is a grid image.
    pub grid_rows: u32,
    /// Grid columns if this is a grid image.
    pub grid_columns: u32,
    /// Whether an auxiliary alpha transparency channel is present.
    pub has_alpha: bool,
    /// Auxiliary alpha item ID if present.
    pub alpha_item_id: Option<u32>,
    /// EXIF metadata item ID if present.
    pub exif_item_id: Option<u32>,
}

impl Default for ContainerMetadata {
    fn default() -> Self {
        Self {
            major_brand: [0; 4],
            compatible_brands: Vec::new(),
            dimensions: ImageDimensions::default(),
            color_space: ColorSpace::Srgb,
            orientation: None,
            primary_item_id: None,
            image_count: 0,
            is_grid: false,
            grid_rows: 0,
            grid_columns: 0,
            has_alpha: false,
            alpha_item_id: None,
            exif_item_id: None,
        }
    }
}
