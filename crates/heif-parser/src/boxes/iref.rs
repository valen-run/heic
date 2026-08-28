//! `iref` (Item Reference Box) parser for `dimg`, `auxl`, `cdsc`, and `thmb`.

use super::{BoxIter, FourCC, FullBoxHeader};
use valen_heic_core::{HeicError, HeicResult, Limits};

/// Single typed item reference link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemReference {
    /// Source item ID that references other items.
    pub from_item_id: u32,
    /// Reference type FourCC (e.g. `dimg`, `auxl`, `cdsc`, `thmb`).
    pub reference_type: FourCC,
    /// Target item IDs referenced by `from_item_id`.
    pub to_item_ids: Vec<u32>,
}

/// Item Reference Box (`iref`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemReferenceBox {
    /// All parsed references.
    pub references: Vec<ItemReference>,
}

impl ItemReferenceBox {
    /// Parses an `iref` box from raw box bytes.
    pub fn parse(input: &[u8], limits: &Limits) -> HeicResult<Self> {
        let full = FullBoxHeader::parse(input)?;
        if full.header.box_type != FourCC::IREF {
            return Err(HeicError::InvalidContainer(format!(
                "Expected 'iref' box, got '{}'",
                full.header.box_type
            )));
        }

        let payload = full.payload(input)?;
        let mut references = Vec::new();

        for res in BoxIter::new(payload) {
            let (ref_header, ref_data) = res?;
            let ref_payload = ref_header.payload(ref_data)?;
            let reference_type = ref_header.box_type;

            let mut cursor;
            let from_item_id = if full.version == 0 {
                if ref_payload.len() < 2 {
                    return Err(HeicError::MalformedInput(
                        "Truncated iref from_item_id".into(),
                    ));
                }
                let id = u16::from_be_bytes([ref_payload[0], ref_payload[1]]) as u32;
                cursor = 2;
                id
            } else {
                if ref_payload.len() < 4 {
                    return Err(HeicError::MalformedInput(
                        "Truncated iref from_item_id".into(),
                    ));
                }
                let id = u32::from_be_bytes([
                    ref_payload[0],
                    ref_payload[1],
                    ref_payload[2],
                    ref_payload[3],
                ]);
                cursor = 4;
                id
            };

            if ref_payload.len() < cursor + 2 {
                return Err(HeicError::MalformedInput(
                    "Truncated iref reference_count".into(),
                ));
            }
            let ref_count =
                u16::from_be_bytes([ref_payload[cursor], ref_payload[cursor + 1]]) as usize;
            cursor += 2;

            limits.check_item_count(ref_count)?;

            let mut to_item_ids = Vec::with_capacity(ref_count);
            for _ in 0..ref_count {
                let to_id = if full.version == 0 {
                    if ref_payload.len() < cursor + 2 {
                        return Err(HeicError::MalformedInput(
                            "Truncated iref to_item_id".into(),
                        ));
                    }
                    let id =
                        u16::from_be_bytes([ref_payload[cursor], ref_payload[cursor + 1]]) as u32;
                    cursor += 2;
                    id
                } else {
                    if ref_payload.len() < cursor + 4 {
                        return Err(HeicError::MalformedInput(
                            "Truncated iref to_item_id".into(),
                        ));
                    }
                    let id = u32::from_be_bytes([
                        ref_payload[cursor],
                        ref_payload[cursor + 1],
                        ref_payload[cursor + 2],
                        ref_payload[cursor + 3],
                    ]);
                    cursor += 4;
                    id
                };
                to_item_ids.push(to_id);
            }

            references.push(ItemReference {
                from_item_id,
                reference_type,
                to_item_ids,
            });
        }

        Ok(Self { references })
    }

    /// Finds derived tile item IDs for a grid item (`dimg` reference).
    pub fn get_derived_image_tiles(&self, grid_item_id: u32) -> Vec<u32> {
        for r in &self.references {
            if r.from_item_id == grid_item_id && r.reference_type == FourCC::DIMG {
                return r.to_item_ids.clone();
            }
        }
        Vec::new()
    }

    /// Finds an auxiliary image item (e.g. alpha plane) that references the given item (`auxl` reference).
    pub fn find_auxiliary_item(&self, primary_item_id: u32) -> Option<u32> {
        for r in &self.references {
            if r.reference_type == FourCC::AUXL && r.to_item_ids.contains(&primary_item_id) {
                return Some(r.from_item_id);
            }
        }
        None
    }

    /// Finds an EXIF metadata item that references the given item (`cdsc` reference).
    pub fn find_exif_item(&self, primary_item_id: u32) -> Option<u32> {
        for r in &self.references {
            if r.reference_type == FourCC::CDSC && r.to_item_ids.contains(&primary_item_id) {
                return Some(r.from_item_id);
            }
        }
        None
    }
}
