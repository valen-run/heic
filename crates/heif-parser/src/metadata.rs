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
        }
    }
}
