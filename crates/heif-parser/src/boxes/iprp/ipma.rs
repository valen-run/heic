//! `ipma` Item Property Association Box parser.

use crate::boxes::FullBoxHeader;
use std::collections::HashMap;
use valen_heic_core::{HeicError, HeicResult, Limits};

/// Item property association table parsed from `ipma`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyAssociation {
    /// Whether this property is essential to interpret the item.
    pub essential: bool,
    /// 1-based index into the `ipco` property array.
    pub property_index: usize,
}

/// Parses associations from `ipma` box.
pub fn parse_ipma(
    input: &[u8],
    limits: &Limits,
) -> HeicResult<HashMap<u32, Vec<PropertyAssociation>>> {
    let full = FullBoxHeader::parse(input)?;
    let payload = full.payload(input)?;

    if payload.len() < 4 {
        return Err(HeicError::MalformedInput("Truncated ipma payload".into()));
    }

    let entry_count = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
    limits.check_item_count(entry_count)?;

    let mut associations = HashMap::with_capacity(entry_count.min(1024));
    let mut cursor = 4;

    for _ in 0..entry_count {
        let item_id = if full.version < 1 {
            if payload.len() < cursor + 2 {
                return Err(HeicError::MalformedInput("Truncated ipma item ID".into()));
            }
            let id = u16::from_be_bytes([payload[cursor], payload[cursor + 1]]) as u32;
            cursor += 2;
            id
        } else {
            if payload.len() < cursor + 4 {
                return Err(HeicError::MalformedInput("Truncated ipma item ID".into()));
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

        if payload.len() < cursor + 1 {
            return Err(HeicError::MalformedInput(
                "Truncated ipma association count".into(),
            ));
        }
        let assoc_count = payload[cursor] as usize;
        cursor += 1;

        let mut item_assocs = Vec::with_capacity(assoc_count);
        for _ in 0..assoc_count {
            if (full.flags & 1) != 0 {
                // 16-bit association
                if payload.len() < cursor + 2 {
                    return Err(HeicError::MalformedInput(
                        "Truncated 16-bit ipma entry".into(),
                    ));
                }
                let raw = u16::from_be_bytes([payload[cursor], payload[cursor + 1]]);
                cursor += 2;
                let essential = (raw & 0x8000) != 0;
                let property_index = (raw & 0x7FFF) as usize;
                item_assocs.push(PropertyAssociation {
                    essential,
                    property_index,
                });
            } else {
                // 8-bit association
                if payload.len() < cursor + 1 {
                    return Err(HeicError::MalformedInput(
                        "Truncated 8-bit ipma entry".into(),
                    ));
                }
                let raw = payload[cursor];
                cursor += 1;
                let essential = (raw & 0x80) != 0;
                let property_index = (raw & 0x7F) as usize;
                item_assocs.push(PropertyAssociation {
                    essential,
                    property_index,
                });
            }
        }

        associations.insert(item_id, item_assocs);
    }

    Ok(associations)
}
