//! Demuxed container construction and item topology resolution.

use super::{HeifFile, ImageItem};
use crate::boxes::{FileTypeBox, FourCC, ImageGrid, ItemPropertiesBox, ItemReferenceBox, MetaBox};
use std::collections::HashMap;
use valen_heic_core::{ColorSpace, HeicError, HeicResult, ImageDimensions, Limits};

impl HeifFile {
    /// Demuxes and builds a [`HeifFile`] from parsed box components.
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

            if dims.width > 0 && dims.height > 0 {
                limits.check_dimensions(dims)?;
            }

            let orientation = iprp
                .get_rotation_for_item(id)
                .map(|r| r.to_exif_orientation());

            let color_space = iprp
                .get_color_for_item(id)
                .map(|c| c.to_color_space())
                .unwrap_or(ColorSpace::Srgb);

            if let Some(loc) = meta.iloc.items.get(&id) {
                if loc.construction_method
                    == crate::boxes::meta::iloc::ConstructionMethod::FileOffset
                {
                    for extent in &loc.extents {
                        let start = usize::try_from(loc.base_offset.saturating_add(extent.offset))
                            .map_err(|_| {
                                HeicError::LimitExceeded("Extent offset exceeds usize".into())
                            })?;
                        let len = usize::try_from(extent.length).map_err(|_| {
                            HeicError::LimitExceeded("Extent length exceeds usize".into())
                        })?;
                        if start.saturating_add(len) > file_bytes.len() {
                            return Err(HeicError::MalformedInput(format!(
                                "Extent [{}..{}] exceeds file length {}",
                                start,
                                start.saturating_add(len),
                                file_bytes.len()
                            )));
                        }
                    }
                }
            }

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
}
