//! Primary image identification, NAL bitstream extraction, and demuxed container models.

pub mod builder;
pub mod extractor;
pub mod item;

pub use item::ImageItem;

use crate::boxes::{FileTypeBox, ImageGrid, ItemPropertiesBox, ItemReferenceBox, MetaBox};
use crate::metadata::ContainerMetadata;
use std::collections::HashMap;
use valen_heic_core::{ColorSpace, ImageDimensions};

/// Fully parsed and demuxed HEIF container representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeifFile {
    /// File type and brand declaration.
    pub ftyp: FileTypeBox,
    /// Primary metadata hierarchy.
    pub meta: MetaBox,
    /// Item properties and associations.
    pub iprp: ItemPropertiesBox,
    /// Item references (`dimg`, `auxl`, `cdsc`).
    pub iref: ItemReferenceBox,
    /// Identified primary image item ID.
    pub primary_item_id: u32,
    /// Grid configuration if the primary image is a `grid` derived item.
    pub grid_config: Option<ImageGrid>,
    /// Tile item IDs in row-major order if this is a grid image.
    pub grid_tile_item_ids: Vec<u32>,
    /// Auxiliary alpha transparency item ID if present.
    pub alpha_item_id: Option<u32>,
    /// EXIF metadata item ID if present.
    pub exif_item_id: Option<u32>,
    /// Map of `item_id -> ImageItem` descriptors.
    pub items: HashMap<u32, ImageItem>,
}

impl HeifFile {
    /// Exports high-level container metadata.
    pub fn get_metadata(&self) -> ContainerMetadata {
        let primary = self.items.get(&self.primary_item_id);
        let dimensions = primary
            .map(|p| p.dimensions)
            .unwrap_or_else(|| ImageDimensions::new(0, 0));
        let color_space = primary
            .map(|p| p.color_space.clone())
            .unwrap_or(ColorSpace::Srgb);
        let orientation = primary.and_then(|p| p.orientation);

        ContainerMetadata {
            major_brand: self.ftyp.major_brand,
            compatible_brands: self.ftyp.compatible_brands.clone(),
            dimensions,
            color_space,
            orientation,
            primary_item_id: Some(self.primary_item_id),
            image_count: self.items.len(),
            is_grid: self.grid_config.is_some(),
            grid_rows: self.grid_config.map(|g| g.rows).unwrap_or(0),
            grid_columns: self.grid_config.map(|g| g.columns).unwrap_or(0),
            has_alpha: self.alpha_item_id.is_some(),
            alpha_item_id: self.alpha_item_id,
            exif_item_id: self.exif_item_id,
        }
    }
}
