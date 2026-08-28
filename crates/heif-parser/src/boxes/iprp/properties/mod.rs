//! Individual item property definitions and `ipco` box parsing.

pub mod color;
pub mod hevc;
pub mod spatial;

pub use color::{AuxiliaryProperty, ColorProperty};
pub use hevc::HevcConfigProperty;
pub use spatial::{ImageSpatialExtents, MirrorProperty, RotationProperty};

use crate::boxes::iprp::hvcc::parse_hvcc;
use crate::boxes::{BoxHeader, BoxIter, FourCC, FullBoxHeader};
use valen_heic_core::{HeicError, HeicResult, Limits};

/// Polymorphic item property representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemProperty {
    /// Image dimensions.
    SpatialExtents(ImageSpatialExtents),
    /// Image rotation.
    Rotation(RotationProperty),
    /// Image flip/mirror.
    Mirror(MirrorProperty),
    /// Color description or ICC profile.
    Color(ColorProperty),
    /// Auxiliary channel declaration (alpha mask).
    Auxiliary(AuxiliaryProperty),
    /// HEVC codec configuration (VPS/SPS/PPS parameter sets).
    HevcConfig(HevcConfigProperty),
    /// Other unparsed property.
    Other(FourCC),
}

/// Parses properties from `ipco` box.
pub fn parse_ipco(input: &[u8], _limits: &Limits) -> HeicResult<Vec<ItemProperty>> {
    let header = BoxHeader::parse(input)?;
    let payload = header.payload(input)?;
    let mut properties = Vec::new();

    for res in BoxIter::new(payload) {
        let (prop_header, prop_data) = res?;
        let prop = match prop_header.box_type {
            FourCC::ISPE => {
                let full = FullBoxHeader::parse(prop_data)?;
                let pl = full.payload(prop_data)?;
                if pl.len() < 8 {
                    return Err(HeicError::MalformedInput("Truncated ispe box".into()));
                }
                let width = u32::from_be_bytes([pl[0], pl[1], pl[2], pl[3]]);
                let height = u32::from_be_bytes([pl[4], pl[5], pl[6], pl[7]]);
                ItemProperty::SpatialExtents(ImageSpatialExtents { width, height })
            }
            FourCC::IROT => {
                let pl = prop_header.payload(prop_data)?;
                if pl.is_empty() {
                    return Err(HeicError::MalformedInput("Truncated irot box".into()));
                }
                let angle_ccw = pl[0] & 3;
                ItemProperty::Rotation(RotationProperty { angle_ccw })
            }
            FourCC::IMIR => {
                let pl = prop_header.payload(prop_data)?;
                if pl.is_empty() {
                    return Err(HeicError::MalformedInput("Truncated imir box".into()));
                }
                let axis = pl[0] & 1;
                ItemProperty::Mirror(MirrorProperty { axis })
            }
            FourCC::COLR => {
                let pl = prop_header.payload(prop_data)?;
                if pl.len() < 4 {
                    return Err(HeicError::MalformedInput("Truncated colr box".into()));
                }
                let colr_type = FourCC([pl[0], pl[1], pl[2], pl[3]]);
                if colr_type == FourCC(*b"nclx") {
                    if pl.len() < 11 {
                        return Err(HeicError::MalformedInput("Truncated nclx colr box".into()));
                    }
                    let colour_primaries = u16::from_be_bytes([pl[4], pl[5]]);
                    let transfer_characteristics = u16::from_be_bytes([pl[6], pl[7]]);
                    let matrix_coefficients = u16::from_be_bytes([pl[8], pl[9]]);
                    let full_range_flag = (pl[10] & 0x80) != 0;
                    ItemProperty::Color(ColorProperty::Nclx {
                        colour_primaries,
                        transfer_characteristics,
                        matrix_coefficients,
                        full_range_flag,
                    })
                } else if colr_type == FourCC(*b"rICC") || colr_type == FourCC(*b"prof") {
                    ItemProperty::Color(ColorProperty::IccProfile(pl[4..].to_vec()))
                } else {
                    ItemProperty::Other(FourCC::COLR)
                }
            }
            FourCC::AUXC => {
                let full = FullBoxHeader::parse(prop_data)?;
                let pl = full.payload(prop_data)?;
                let mut cursor = 0;
                let mut end = 0;
                while end < pl.len() && pl[end] != 0 {
                    end += 1;
                }
                let aux_type = String::from_utf8_lossy(&pl[cursor..end]).to_string();
                if end < pl.len() {
                    cursor = end + 1;
                } else {
                    cursor = end;
                }
                let sub_type = pl[cursor..].to_vec();
                ItemProperty::Auxiliary(AuxiliaryProperty { aux_type, sub_type })
            }
            FourCC::HVCC => {
                let pl = prop_header.payload(prop_data)?;
                let hvc = parse_hvcc(pl)?;
                ItemProperty::HevcConfig(hvc)
            }
            other => ItemProperty::Other(other),
        };
        properties.push(prop);
    }

    Ok(properties)
}
