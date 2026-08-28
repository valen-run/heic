//! `iinf` (Item Info Box) and `infe` parser.

use crate::boxes::{BoxIter, FourCC, FullBoxHeader};
use std::collections::HashMap;
use valen_heic_core::{HeicError, HeicResult, Limits};

/// Information entry for an individual item parsed from `infe`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemInfoEntry {
    /// Item identifier.
    pub item_id: u32,
    /// Protection index (0 for unencrypted).
    pub item_protection_index: u16,
    /// FourCC item type (e.g. `hvc1`, `grid`, `Exif`, `mime`).
    pub item_type: FourCC,
    /// Human-readable item name if present.
    pub item_name: String,
    /// MIME content type for `mime` items.
    pub content_type: Option<String>,
}

/// Item Information Box (`iinf`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemInfo {
    /// Map of `item_id -> ItemInfoEntry`.
    pub entries: HashMap<u32, ItemInfoEntry>,
}

impl ItemInfo {
    /// Parses an `iinf` box and all its child `infe` boxes.
    pub fn parse(input: &[u8], limits: &Limits) -> HeicResult<Self> {
        let full = FullBoxHeader::parse(input)?;
        if full.header.box_type != FourCC::IINF {
            return Err(HeicError::InvalidContainer(format!(
                "Expected 'iinf' box, got '{}'",
                full.header.box_type
            )));
        }

        let payload = full.payload(input)?;
        let (entry_count, cursor) = if full.version == 0 {
            if payload.len() < 2 {
                return Err(HeicError::MalformedInput(
                    "Truncated iinf entry count".into(),
                ));
            }
            let count = u16::from_be_bytes([payload[0], payload[1]]) as usize;
            (count, 2)
        } else {
            if payload.len() < 4 {
                return Err(HeicError::MalformedInput(
                    "Truncated iinf entry count".into(),
                ));
            }
            let count =
                u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
            (count, 4)
        };

        limits.check_item_count(entry_count)?;

        let mut entries = HashMap::with_capacity(entry_count.min(1024));

        let child_data = &payload[cursor..];
        for res in BoxIter::new(child_data) {
            let (header, box_data) = res?;
            if header.box_type == FourCC::INFE {
                let entry = parse_infe(box_data)?;
                entries.insert(entry.item_id, entry);
            }
        }

        Ok(Self { entries })
    }
}

/// Parses a single `infe` item info entry box.
pub fn parse_infe(input: &[u8]) -> HeicResult<ItemInfoEntry> {
    let full = FullBoxHeader::parse(input)?;
    let payload = full.payload(input)?;

    let mut cursor;
    if full.version == 2 {
        if payload.len() < 8 {
            return Err(HeicError::MalformedInput(
                "Truncated version 2 infe box".into(),
            ));
        }
        let item_id = u16::from_be_bytes([payload[0], payload[1]]) as u32;
        let item_protection_index = u16::from_be_bytes([payload[2], payload[3]]);
        let item_type = FourCC([payload[4], payload[5], payload[6], payload[7]]);
        cursor = 8;

        let item_name = read_null_terminated_string(payload, &mut cursor)?;
        let content_type = if item_type == FourCC::MIME {
            Some(read_null_terminated_string(payload, &mut cursor)?)
        } else {
            None
        };

        Ok(ItemInfoEntry {
            item_id,
            item_protection_index,
            item_type,
            item_name,
            content_type,
        })
    } else if full.version == 3 {
        if payload.len() < 10 {
            return Err(HeicError::MalformedInput(
                "Truncated version 3 infe box".into(),
            ));
        }
        let item_id = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
        let item_protection_index = u16::from_be_bytes([payload[4], payload[5]]);
        let item_type = FourCC([payload[6], payload[7], payload[8], payload[9]]);
        cursor = 10;

        let item_name = read_null_terminated_string(payload, &mut cursor)?;
        let content_type = if item_type == FourCC::MIME {
            Some(read_null_terminated_string(payload, &mut cursor)?)
        } else {
            None
        };

        Ok(ItemInfoEntry {
            item_id,
            item_protection_index,
            item_type,
            item_name,
            content_type,
        })
    } else {
        // Version 0 or 1
        if payload.len() < 4 {
            return Err(HeicError::MalformedInput(
                "Truncated version 0/1 infe box".into(),
            ));
        }
        let item_id = u16::from_be_bytes([payload[0], payload[1]]) as u32;
        let item_protection_index = u16::from_be_bytes([payload[2], payload[3]]);
        cursor = 4;
        let item_name = read_null_terminated_string(payload, &mut cursor)?;
        let content_type = if cursor < payload.len() {
            Some(read_null_terminated_string(payload, &mut cursor)?)
        } else {
            None
        };

        Ok(ItemInfoEntry {
            item_id,
            item_protection_index,
            item_type: FourCC::HVC1, // Default fallback for v0
            item_name,
            content_type,
        })
    }
}

/// Helper to read a null-terminated UTF-8 string from a byte slice.
pub fn read_null_terminated_string(data: &[u8], cursor: &mut usize) -> HeicResult<String> {
    let start = *cursor;
    let mut end = start;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }

    let s = String::from_utf8_lossy(&data[start..end]).to_string();
    if end < data.len() {
        *cursor = end + 1; // skip null byte
    } else {
        *cursor = end;
    }
    Ok(s)
}
