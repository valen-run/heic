//! Primary image identification, NAL bitstream extraction, and demuxed container models.

use crate::boxes::{
    ConstructionMethod, FileTypeBox, FourCC, ImageGrid, ItemPropertiesBox, ItemReferenceBox,
    MetaBox,
};
use crate::metadata::ContainerMetadata;
use std::collections::HashMap;
use valen_heic_core::{ColorSpace, HeicError, HeicResult, ImageDimensions, Limits};

/// Descriptor of a single image item inside a HEIF container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageItem {
    /// Item identifier.
    pub id: u32,
    /// Whether this is the primary item of the container.
    pub is_primary: bool,
    /// Item FourCC type (e.g. `hvc1`, `grid`, `Exif`, `mime`).
    pub item_type: FourCC,
    /// Dimensions of the image item.
    pub dimensions: ImageDimensions,
    /// EXIF orientation tag if present.
    pub orientation: Option<u8>,
    /// Color space or profile.
    pub color_space: ColorSpace,
    /// Total data length in bytes.
    pub length: u64,
    /// Whether this item represents an auxiliary alpha mask.
    pub is_alpha_mask: bool,
}

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
    /// Demuxes and builds a [`HeifFile`] from parsed box components.
    ///
    /// Pipeline:
    /// 1. Identifies the primary item ID (via `pitm` box or fallback to first `hvc1`/`grid` item in `iinf`).
    /// 2. Discovers auxiliary alpha masks via `iref` (`auxl` reference) or `auxC` property tags.
    /// 3. Locates EXIF metadata via `iref` (`cdsc` reference) or `Exif` item entries.
    /// 4. Parses `grid` layout and tile associations if the primary item is a derived grid image.
    /// 5. Populates [`ImageItem`] descriptors with dimensions, color profiles, rotations, and lengths.
    pub fn build(
        ftyp: FileTypeBox,
        meta: MetaBox,
        iprp: ItemPropertiesBox,
        iref: ItemReferenceBox,
        file_bytes: &[u8],
        limits: &Limits,
    ) -> HeicResult<Self> {
        let primary_item_id = meta
            .primary_item_id
            .or_else(|| {
                // If pitm was not present, fall back to first image item in iinf
                for (&id, entry) in &meta.iinf.entries {
                    if entry.item_type == FourCC::HVC1 || entry.item_type == FourCC::GRID {
                        return Some(id);
                    }
                }
                meta.iinf.entries.keys().next().copied()
            })
            .ok_or_else(|| {
                HeicError::InvalidContainer("No primary item found in HEIF container".into())
            })?;

        let alpha_item_id = iref.find_auxiliary_item(primary_item_id).or_else(|| {
            // Or search for any item with auxC property matching alpha
            meta.iinf
                .entries
                .keys()
                .copied()
                .find(|&id| id != primary_item_id && iprp.is_alpha_mask_item(id))
        });

        let exif_item_id = iref.find_exif_item(primary_item_id).or_else(|| {
            meta.iinf
                .entries
                .iter()
                .find(|(_, entry)| entry.item_type == FourCC::EXIF)
                .map(|(&id, _)| id)
        });

        // Inspect primary item type
        let primary_type = meta
            .iinf
            .entries
            .get(&primary_item_id)
            .map(|e| e.item_type)
            .unwrap_or(FourCC::HVC1);

        let (grid_config, grid_tile_item_ids) = if primary_type == FourCC::GRID {
            let file_temp = Self {
                ftyp: ftyp.clone(),
                meta: meta.clone(),
                iprp: iprp.clone(),
                iref: iref.clone(),
                primary_item_id,
                grid_config: None,
                grid_tile_item_ids: Vec::new(),
                alpha_item_id,
                exif_item_id,
                items: HashMap::new(),
            };

            let grid_payload = file_temp.extract_item_data(file_bytes, primary_item_id)?;
            let grid = ImageGrid::parse(&grid_payload, limits)?;
            let tiles = iref.get_derived_image_tiles(primary_item_id);

            if !tiles.is_empty() && tiles.len() != grid.tile_count() as usize {
                return Err(HeicError::InvalidContainer(format!(
                    "Grid declares {} tiles but iref contains {} tile references",
                    grid.tile_count(),
                    tiles.len()
                )));
            }

            (Some(grid), tiles)
        } else {
            (None, Vec::new())
        };

        // Construct ImageItem descriptors
        let mut items = HashMap::with_capacity(meta.iinf.entries.len());
        for (&id, entry) in &meta.iinf.entries {
            let is_primary = id == primary_item_id;
            let mut dims = iprp
                .get_dimensions_for_item(id)
                .unwrap_or_else(|| ImageDimensions::new(0, 0));

            if is_primary {
                if let Some(ref grid) = grid_config {
                    dims = grid.output_dimensions();
                }
            }

            let orientation = iprp
                .get_rotation_for_item(id)
                .map(|r| r.to_exif_orientation());

            let color_space = iprp
                .get_color_for_item(id)
                .map(|c| c.to_color_space())
                .unwrap_or(ColorSpace::Srgb);

            let length = meta
                .iloc
                .items
                .get(&id)
                .map(|loc| loc.total_length())
                .unwrap_or(0);

            let is_alpha_mask = Some(id) == alpha_item_id || iprp.is_alpha_mask_item(id);

            items.insert(
                id,
                ImageItem {
                    id,
                    is_primary,
                    item_type: entry.item_type,
                    dimensions: dims,
                    orientation,
                    color_space,
                    length,
                    is_alpha_mask,
                },
            );
        }

        Ok(Self {
            ftyp,
            meta,
            iprp,
            iref,
            primary_item_id,
            grid_config,
            grid_tile_item_ids,
            alpha_item_id,
            exif_item_id,
            items,
        })
    }

    /// Extracts raw payload data for an item by reading all its extents according to `iloc`.
    pub fn extract_item_data(&self, file_bytes: &[u8], item_id: u32) -> HeicResult<Vec<u8>> {
        let loc = self.meta.iloc.items.get(&item_id).ok_or_else(|| {
            HeicError::InvalidContainer(format!("Item ID {item_id} has no iloc entry"))
        })?;

        let mut output = Vec::with_capacity(usize::try_from(loc.total_length()).unwrap_or(0));

        match loc.construction_method {
            ConstructionMethod::FileOffset => {
                for extent in &loc.extents {
                    let start = usize::try_from(loc.base_offset.saturating_add(extent.offset))
                        .map_err(|_| {
                            HeicError::LimitExceeded("Extent offset exceeds usize".into())
                        })?;
                    let len = usize::try_from(extent.length).map_err(|_| {
                        HeicError::LimitExceeded("Extent length exceeds usize".into())
                    })?;
                    let end = start.saturating_add(len);

                    if end > file_bytes.len() {
                        return Err(HeicError::MalformedInput(format!(
                            "Extent [{}..{}] exceeds file length {}",
                            start,
                            end,
                            file_bytes.len()
                        )));
                    }
                    output.extend_from_slice(&file_bytes[start..end]);
                }
            }
            ConstructionMethod::IdatOffset => {
                let idat_data = self.meta.idat.as_ref().ok_or_else(|| {
                    HeicError::InvalidContainer(
                        "Item uses idat construction but meta has no idat box".into(),
                    )
                })?;

                for extent in &loc.extents {
                    let start = usize::try_from(loc.base_offset.saturating_add(extent.offset))
                        .map_err(|_| {
                            HeicError::LimitExceeded("Extent offset exceeds usize".into())
                        })?;
                    let len = usize::try_from(extent.length).map_err(|_| {
                        HeicError::LimitExceeded("Extent length exceeds usize".into())
                    })?;
                    let end = start.saturating_add(len);

                    if end > idat_data.len() {
                        return Err(HeicError::MalformedInput(format!(
                            "Extent [{}..{}] exceeds idat length {}",
                            start,
                            end,
                            idat_data.len()
                        )));
                    }
                    output.extend_from_slice(&idat_data[start..end]);
                }
            }
            ConstructionMethod::ItemOffset => {
                return Err(HeicError::UnsupportedFeature(
                    "ConstructionMethod 2 (item offset) is not yet supported".to_string(),
                ));
            }
        }

        Ok(output)
    }

    /// Extracts an Annex-B formatted HEVC bitstream (SPS/PPS/VPS headers + length-prefixed slice NAL units).
    pub fn extract_annex_b_stream(&self, file_bytes: &[u8], item_id: u32) -> HeicResult<Vec<u8>> {
        let item_data = self.extract_item_data(file_bytes, item_id)?;
        let hevc_config = self.iprp.get_hevc_config_for_item(item_id);

        let mut annex_b = Vec::new();

        let nalu_length_size = if let Some(config) = hevc_config {
            annex_b.extend_from_slice(&config.to_annex_b_header());
            config.nalu_length_size as usize
        } else {
            4 // Standard default HEVC length size
        };

        let mut offset = 0;
        while offset + nalu_length_size <= item_data.len() {
            let nalu_len = match nalu_length_size {
                1 => item_data[offset] as usize,
                2 => u16::from_be_bytes([item_data[offset], item_data[offset + 1]]) as usize,
                4 => u32::from_be_bytes([
                    item_data[offset],
                    item_data[offset + 1],
                    item_data[offset + 2],
                    item_data[offset + 3],
                ]) as usize,
                _ => 4,
            };
            offset += nalu_length_size;

            if offset + nalu_len > item_data.len() {
                return Err(HeicError::MalformedInput(format!(
                    "NAL unit length {} exceeds remaining slice data at offset {}",
                    nalu_len, offset
                )));
            }

            annex_b.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
            annex_b.extend_from_slice(&item_data[offset..offset + nalu_len]);
            offset += nalu_len;
        }

        Ok(annex_b)
    }

    /// Extracts raw EXIF metadata bytes if present in the container.
    pub fn extract_exif_data(&self, file_bytes: &[u8]) -> HeicResult<Option<Vec<u8>>> {
        let Some(exif_id) = self.exif_item_id else {
            return Ok(None);
        };

        let raw = self.extract_item_data(file_bytes, exif_id)?;
        if raw.len() >= 4 {
            // In HEIF, EXIF items often start with a 4-byte offset indicating where TIFF header starts
            let exif_offset = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]) as usize;
            let start = 4 + exif_offset;
            if start < raw.len() {
                return Ok(Some(raw[start..].to_vec()));
            }
        }

        Ok(Some(raw))
    }

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
