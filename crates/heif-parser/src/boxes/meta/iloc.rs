//! `iloc` Item Location Box parser.

use crate::boxes::{FourCC, FullBoxHeader};
use std::collections::HashMap;
use valen_heic_core::{HeicError, HeicResult, Limits};

/// Extent location descriptor within a file or `idat` payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemExtent {
    /// Extent byte offset (relative to base offset).
    pub offset: u64,
    /// Extent byte length.
    pub length: u64,
    /// Extent index if indexed (version 1/2).
    pub index: Option<u64>,
}

/// Item construction method declared in `iloc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConstructionMethod {
    /// 0 = File offset (relative to file start / `mdat`).
    #[default]
    FileOffset = 0,
    /// 1 = `idat` offset (relative to `idat` box payload).
    IdatOffset = 1,
    /// 2 = Item offset (relative to another item payload).
    ItemOffset = 2,
}

/// Item location information parsed from `iloc`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemLocation {
    /// Item identifier.
    pub item_id: u32,
    /// Construction method.
    pub construction_method: ConstructionMethod,
    /// Data reference index (typically 0 for local file).
    pub data_reference_index: u16,
    /// Base offset added to all extent offsets.
    pub base_offset: u64,
    /// List of extents composing this item's data.
    pub extents: Vec<ItemExtent>,
}

impl ItemLocation {
    /// Computes the total byte length across all extents.
    pub fn total_length(&self) -> u64 {
        self.extents.iter().map(|e| e.length).sum()
    }
}

/// Item location table (`iloc` box).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemLocationBox {
    /// Map of `item_id -> ItemLocation`.
    pub items: HashMap<u32, ItemLocation>,
}

impl ItemLocationBox {
    /// Parses an `iloc` box from raw box bytes.
    pub fn parse(input: &[u8], limits: &Limits) -> HeicResult<Self> {
        let full = FullBoxHeader::parse(input)?;
        if full.header.box_type != FourCC::ILOC {
            return Err(HeicError::InvalidContainer(format!(
                "Expected 'iloc' box, got '{}'",
                full.header.box_type
            )));
        }

        let payload = full.payload(input)?;
        if payload.len() < 2 {
            return Err(HeicError::MalformedInput(
                "Insufficient bytes for iloc size descriptors".to_string(),
            ));
        }

        let offset_size = (payload[0] >> 4) as usize;
        let length_size = (payload[0] & 0x0F) as usize;
        let base_offset_size = (payload[1] >> 4) as usize;
        let index_size = if full.version == 1 || full.version == 2 {
            (payload[1] & 0x0F) as usize
        } else {
            0
        };

        let mut cursor = 2;
        let item_count = if full.version < 2 {
            if payload.len() < cursor + 2 {
                return Err(HeicError::MalformedInput(
                    "Truncated iloc item count".into(),
                ));
            }
            let count = u16::from_be_bytes([payload[cursor], payload[cursor + 1]]) as usize;
            cursor += 2;
            count
        } else {
            if payload.len() < cursor + 4 {
                return Err(HeicError::MalformedInput(
                    "Truncated iloc item count".into(),
                ));
            }
            let count = u32::from_be_bytes([
                payload[cursor],
                payload[cursor + 1],
                payload[cursor + 2],
                payload[cursor + 3],
            ]) as usize;
            cursor += 4;
            count
        };

        limits.check_item_count(item_count)?;

        let mut items = HashMap::with_capacity(item_count.min(1024));

        for _ in 0..item_count {
            let item_id = if full.version < 2 {
                if payload.len() < cursor + 2 {
                    return Err(HeicError::MalformedInput("Truncated iloc item ID".into()));
                }
                let id = u16::from_be_bytes([payload[cursor], payload[cursor + 1]]) as u32;
                cursor += 2;
                id
            } else {
                if payload.len() < cursor + 4 {
                    return Err(HeicError::MalformedInput("Truncated iloc item ID".into()));
                }
                let id = u32::from_be_bytes([
                    payload[cursor],
                    payload[cursor + 1],
                    payload[cursor + 2],
                    payload[cursor + 3],
                ]);
                cursor += 4;
                id
            };

            let construction_method = if full.version == 1 || full.version == 2 {
                if payload.len() < cursor + 2 {
                    return Err(HeicError::MalformedInput(
                        "Truncated iloc construction method".into(),
                    ));
                }
                let method_raw = payload[cursor + 1] & 0x0F;
                cursor += 2;
                match method_raw {
                    0 => ConstructionMethod::FileOffset,
                    1 => ConstructionMethod::IdatOffset,
                    2 => ConstructionMethod::ItemOffset,
                    _ => ConstructionMethod::FileOffset,
                }
            } else {
                ConstructionMethod::FileOffset
            };

            if payload.len() < cursor + 2 {
                return Err(HeicError::MalformedInput(
                    "Truncated iloc data_reference_index".into(),
                ));
            }
            let data_reference_index = u16::from_be_bytes([payload[cursor], payload[cursor + 1]]);
            cursor += 2;

            let base_offset = read_variable_uint(payload, &mut cursor, base_offset_size)?;

            if payload.len() < cursor + 2 {
                return Err(HeicError::MalformedInput(
                    "Truncated iloc extent_count".into(),
                ));
            }
            let extent_count = u16::from_be_bytes([payload[cursor], payload[cursor + 1]]) as usize;
            cursor += 2;

            let mut extents = Vec::with_capacity(extent_count);
            for _ in 0..extent_count {
                let index = if (full.version == 1 || full.version == 2) && index_size > 0 {
                    Some(read_variable_uint(payload, &mut cursor, index_size)?)
                } else {
                    None
                };

                let offset = read_variable_uint(payload, &mut cursor, offset_size)?;
                let length = read_variable_uint(payload, &mut cursor, length_size)?;

                extents.push(ItemExtent {
                    offset,
                    length,
                    index,
                });
            }

            items.insert(
                item_id,
                ItemLocation {
                    item_id,
                    construction_method,
                    data_reference_index,
                    base_offset,
                    extents,
                },
            );
        }

        Ok(Self { items })
    }
}

/// Helper function to read a variable-length big-endian unsigned integer (1, 2, 4, or 8 bytes).
pub fn read_variable_uint(data: &[u8], cursor: &mut usize, size: usize) -> HeicResult<u64> {
    if size == 0 {
        return Ok(0);
    }
    if data.len() < *cursor + size {
        return Err(HeicError::MalformedInput(format!(
            "Insufficient bytes for variable integer of size {size}"
        )));
    }

    let val = match size {
        1 => data[*cursor] as u64,
        2 => u16::from_be_bytes([data[*cursor], data[*cursor + 1]]) as u64,
        4 => u32::from_be_bytes([
            data[*cursor],
            data[*cursor + 1],
            data[*cursor + 2],
            data[*cursor + 3],
        ]) as u64,
        8 => u64::from_be_bytes([
            data[*cursor],
            data[*cursor + 1],
            data[*cursor + 2],
            data[*cursor + 3],
            data[*cursor + 4],
            data[*cursor + 5],
            data[*cursor + 6],
            data[*cursor + 7],
        ]),
        other => {
            return Err(HeicError::InvalidContainer(format!(
                "Unsupported variable integer size: {other}"
            )))
        }
    };

    *cursor += size;
    Ok(val)
}
