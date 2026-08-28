//! Individual item property definitions (`ispe`, `irot`, `imir`, `colr`, `auxC`, `hvcC`).

use crate::boxes::iprp::hvcc::parse_hvcc;
use crate::boxes::{BoxHeader, BoxIter, FourCC, FullBoxHeader};
use valen_heic_core::{ColorSpace, HeicError, HeicResult, ImageDimensions, Limits};

/// Image spatial extents parsed from `ispe`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageSpatialExtents {
    /// Display width in pixels.
    pub width: u32,
    /// Display height in pixels.
    pub height: u32,
}

impl ImageSpatialExtents {
    /// Converts extents to core [`ImageDimensions`].
    pub const fn to_dimensions(&self) -> ImageDimensions {
        ImageDimensions::new(self.width, self.height)
    }
}

/// Image rotation parsed from `irot`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RotationProperty {
    /// Rotation angle in degrees CCW (0 = 0°, 1 = 90° CCW / 270° CW, 2 = 180°, 3 = 270° CCW / 90° CW).
    pub angle_ccw: u8,
}

impl RotationProperty {
    /// Returns rotation angle in degrees clockwise (0, 90, 180, 270).
    pub const fn angle_cw(&self) -> u16 {
        match self.angle_ccw & 3 {
            0 => 0,
            1 => 270,
            2 => 180,
            3 => 90,
            _ => 0,
        }
    }

    /// Converts to EXIF orientation tag if pure rotation (tag 1, 3, 6, 8).
    pub const fn to_exif_orientation(&self) -> u8 {
        match self.angle_ccw & 3 {
            0 => 1, // Normal
            1 => 8, // 270 CW
            2 => 3, // 180
            3 => 6, // 90 CW
            _ => 1,
        }
    }
}

/// Image mirror parsed from `imir`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MirrorProperty {
    /// 0 = vertical axis (left-right flip), 1 = horizontal axis (top-bottom flip).
    pub axis: u8,
}

/// Colour information parsed from `colr`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorProperty {
    /// NCLX colour profile description.
    Nclx {
        /// Colour primaries index.
        colour_primaries: u16,
        /// Transfer characteristics index.
        transfer_characteristics: u16,
        /// Matrix coefficients index.
        matrix_coefficients: u16,
        /// Full range video flag.
        full_range_flag: bool,
    },
    /// Embedded raw ICC profile data (`rICC` or `prof`).
    IccProfile(Vec<u8>),
}

impl ColorProperty {
    /// Converts to core [`ColorSpace`].
    pub fn to_color_space(&self) -> ColorSpace {
        match self {
            Self::Nclx {
                colour_primaries, ..
            } => match colour_primaries {
                1 => ColorSpace::Srgb,
                12 => ColorSpace::DisplayP3,
                9 => ColorSpace::Rec2020,
                _ => ColorSpace::Srgb,
            },
            Self::IccProfile(bytes) => ColorSpace::IccProfile(bytes.clone()),
        }
    }
}

/// Auxiliary image type parsed from `auxC`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuxiliaryProperty {
    /// URN identifier of auxiliary stream (e.g. `urn:mpeg:hevc:2015:auxid:1` for alpha channel).
    pub aux_type: String,
    /// Sub-type configuration data.
    pub sub_type: Vec<u8>,
}

impl AuxiliaryProperty {
    /// Returns `true` if this auxiliary property identifies an alpha transparency mask.
    pub fn is_alpha(&self) -> bool {
        self.aux_type == "urn:mpeg:hevc:2015:auxid:1"
            || self.aux_type.ends_with(":auxid:1")
            || self.aux_type.eq_ignore_ascii_case("alpha")
    }
}

/// HEVC configuration record parsed from `hvcC`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HevcConfigProperty {
    /// NAL unit length size in bytes (typically 4).
    pub nalu_length_size: u8,
    /// Sequence Parameter Sets (SPS).
    pub sps: Vec<Vec<u8>>,
    /// Picture Parameter Sets (PPS).
    pub pps: Vec<Vec<u8>>,
    /// Video Parameter Sets (VPS).
    pub vps: Vec<Vec<u8>>,
}

impl HevcConfigProperty {
    /// Formats all VPS, SPS, and PPS parameter sets into Annex-B start-code prefixed bitstream bytes.
    pub fn to_annex_b_header(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for vps in &self.vps {
            out.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
            out.extend_from_slice(vps);
        }
        for sps in &self.sps {
            out.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
            out.extend_from_slice(sps);
        }
        for pps in &self.pps {
            out.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
            out.extend_from_slice(pps);
        }
        out
    }
}

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
