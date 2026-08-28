//! ISOBMFF Box header structures and parsing primitives.

pub mod fourcc;
pub mod ftyp;
pub mod grid;
pub mod header;
pub mod iprp;
pub mod iref;
pub mod iter;
pub mod meta;

pub use fourcc::FourCC;
pub use ftyp::FileTypeBox;
pub use grid::ImageGrid;
pub use header::{BoxHeader, FullBoxHeader};
pub use iprp::{
    AuxiliaryProperty, ColorProperty, HevcConfigProperty, ImageSpatialExtents, ItemPropertiesBox,
    ItemProperty, MirrorProperty, RotationProperty,
};
pub use iref::{ItemReference, ItemReferenceBox};
pub use iter::BoxIter;
pub use meta::{
    ConstructionMethod, ItemInfo, ItemInfoEntry, ItemLocation, ItemLocationBox, MetaBox,
};

#[cfg(test)]
mod tests {
    use super::*;
    use valen_heic_core::HeicResult;

    #[test]
    fn test_parse_standard_header() {
        let raw = [0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p'];
        let header = BoxHeader::parse(&raw).unwrap();
        assert_eq!(header.box_type, FourCC::FTYP);
        assert_eq!(header.size, 24);
        assert_eq!(header.header_size, 8);
        assert_eq!(header.payload_size(), 16);
    }

    #[test]
    fn test_parse_extended_header() {
        let mut raw = vec![0x00, 0x00, 0x00, 0x01, b'm', b'd', b'a', b't'];
        raw.extend_from_slice(&1000u64.to_be_bytes());
        raw.extend_from_slice(&vec![0u8; 1000 - 16]);

        let header = BoxHeader::parse(&raw).unwrap();
        assert_eq!(header.box_type, FourCC::MDAT);
        assert_eq!(header.size, 1000);
        assert_eq!(header.header_size, 16);
        assert_eq!(header.payload_size(), 984);
    }

    #[test]
    fn test_box_iter() {
        let mut data = Vec::new();
        // Box 1: ftyp (16 bytes)
        data.extend_from_slice(&[0, 0, 0, 16, b'f', b't', b'y', b'p', 0, 0, 0, 0, 0, 0, 0, 0]);
        // Box 2: meta (12 bytes)
        data.extend_from_slice(&[0, 0, 0, 12, b'm', b'e', b't', b'a', 0, 0, 0, 0]);

        let boxes: Vec<_> = BoxIter::new(&data).collect::<HeicResult<Vec<_>>>().unwrap();
        assert_eq!(boxes.len(), 2);
        assert_eq!(boxes[0].0.box_type, FourCC::FTYP);
        assert_eq!(boxes[1].0.box_type, FourCC::META);
    }
}
