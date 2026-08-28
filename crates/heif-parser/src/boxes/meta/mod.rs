//! `meta` (Metadata Box), `hdlr`, `pitm`, `iinf`/`infe`, `iloc`, and `idat` parser.

pub mod iinf;
pub mod iloc;

use crate::boxes::{BoxIter, FourCC, FullBoxHeader};
use valen_heic_core::{HeicError, HeicResult, Limits};

pub use iinf::{parse_infe, read_null_terminated_string, ItemInfo, ItemInfoEntry};
pub use iloc::{read_variable_uint, ConstructionMethod, ItemExtent, ItemLocation, ItemLocationBox};

/// The complete `meta` (Metadata Box) containing item hierarchy and descriptors.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetaBox {
    /// Handler type parsed from `hdlr` (e.g. `pict`).
    pub handler_type: Option<FourCC>,
    /// Primary item ID parsed from `pitm`.
    pub primary_item_id: Option<u32>,
    /// Item locations parsed from `iloc`.
    pub iloc: ItemLocationBox,
    /// Item information parsed from `iinf`.
    pub iinf: ItemInfo,
    /// Item properties parsed from `iprp` if present.
    pub iprp: Option<super::iprp::ItemPropertiesBox>,
    /// Item references parsed from `iref` if present.
    pub iref: Option<super::iref::ItemReferenceBox>,
    /// Embedded item data parsed from `idat` if present.
    pub idat: Option<Vec<u8>>,
}

impl MetaBox {
    /// Parses a `meta` box from raw box bytes.
    pub fn parse(input: &[u8], limits: &Limits) -> HeicResult<Self> {
        let full = FullBoxHeader::parse(input)?;
        if full.header.box_type != FourCC::META {
            return Err(HeicError::InvalidContainer(format!(
                "Expected 'meta' box, got '{}'",
                full.header.box_type
            )));
        }

        let payload = full.payload(input)?;
        let mut meta = Self::default();

        for res in BoxIter::new(payload) {
            let (header, box_data) = res?;
            match header.box_type {
                FourCC::HDLR => {
                    let full_hdlr = FullBoxHeader::parse(box_data)?;
                    let hdlr_payload = full_hdlr.payload(box_data)?;
                    if hdlr_payload.len() >= 8 {
                        let htype = FourCC([
                            hdlr_payload[4],
                            hdlr_payload[5],
                            hdlr_payload[6],
                            hdlr_payload[7],
                        ]);
                        meta.handler_type = Some(htype);
                    }
                }
                FourCC::PITM => {
                    let full_pitm = FullBoxHeader::parse(box_data)?;
                    let pitm_payload = full_pitm.payload(box_data)?;
                    if full_pitm.version == 0 {
                        if pitm_payload.len() >= 2 {
                            meta.primary_item_id =
                                Some(u16::from_be_bytes([pitm_payload[0], pitm_payload[1]]) as u32);
                        }
                    } else if pitm_payload.len() >= 4 {
                        meta.primary_item_id = Some(u32::from_be_bytes([
                            pitm_payload[0],
                            pitm_payload[1],
                            pitm_payload[2],
                            pitm_payload[3],
                        ]));
                    }
                }
                FourCC::ILOC => {
                    meta.iloc = ItemLocationBox::parse(box_data, limits)?;
                }
                FourCC::IINF => {
                    meta.iinf = ItemInfo::parse(box_data, limits)?;
                }
                FourCC::IPRP => {
                    meta.iprp = Some(super::iprp::ItemPropertiesBox::parse(box_data, limits)?);
                }
                FourCC::IREF => {
                    meta.iref = Some(super::iref::ItemReferenceBox::parse(box_data, limits)?);
                }
                FourCC::IDAT => {
                    let idat_payload = header.payload(box_data)?;
                    meta.idat = Some(idat_payload.to_vec());
                }
                _ => {}
            }
        }

        Ok(meta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitm_parsing() {
        let mut raw = vec![0, 0, 0, 14, b'p', b'i', b't', b'm'];
        raw.extend_from_slice(&[0, 0, 0, 0]); // FullBox v0, flags 0
        raw.extend_from_slice(&1u16.to_be_bytes()); // item_id 1

        let full = FullBoxHeader::parse(&raw).unwrap();
        let payload = full.payload(&raw).unwrap();
        let item_id = u16::from_be_bytes([payload[0], payload[1]]) as u32;
        assert_eq!(item_id, 1);
    }
}
