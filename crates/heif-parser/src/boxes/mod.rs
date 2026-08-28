//! ISOBMFF Box header structures and parsing primitives.

use std::fmt;
use valen_heic_core::{HeicError, HeicResult};

pub mod ftyp;
pub mod grid;
pub mod iprp;
pub mod iref;
pub mod meta;

pub use ftyp::FileTypeBox;
pub use grid::ImageGrid;
pub use iprp::{
    AuxiliaryProperty, ColorProperty, HevcConfigProperty, ImageSpatialExtents, ItemPropertiesBox,
    ItemProperty, MirrorProperty, RotationProperty,
};
pub use iref::{ItemReference, ItemReferenceBox};
pub use meta::{
    ConstructionMethod, ItemInfo, ItemInfoEntry, ItemLocation, ItemLocationBox, MetaBox,
};

/// Four-character box type identifier (FourCC).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FourCC(pub [u8; 4]);

impl FourCC {
    /// Creates a FourCC from a 4-byte slice.
    #[inline]
    pub const fn from_bytes(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }

    /// Returns the underlying 4-byte array reference.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }

    // Top-level containers & metadata boxes
    /// File type box `ftyp`.
    pub const FTYP: Self = Self(*b"ftyp");
    /// Meta box `meta`.
    pub const META: Self = Self(*b"meta");
    /// Handler reference box `hdlr`.
    pub const HDLR: Self = Self(*b"hdlr");
    /// Primary item box `pitm`.
    pub const PITM: Self = Self(*b"pitm");
    /// Item location box `iloc`.
    pub const ILOC: Self = Self(*b"iloc");
    /// Item information box `iinf`.
    pub const IINF: Self = Self(*b"iinf");
    /// Item info entry box `infe`.
    pub const INFE: Self = Self(*b"infe");
    /// Item reference box `iref`.
    pub const IREF: Self = Self(*b"iref");
    /// Media data box `mdat`.
    pub const MDAT: Self = Self(*b"mdat");
    /// Item data box `idat`.
    pub const IDAT: Self = Self(*b"idat");

    // Item properties
    /// Item properties box `iprp`.
    pub const IPRP: Self = Self(*b"iprp");
    /// Item property container box `ipco`.
    pub const IPCO: Self = Self(*b"ipco");
    /// Item property association box `ipma`.
    pub const IPMA: Self = Self(*b"ipma");
    /// Image spatial extents box `ispe`.
    pub const ISPE: Self = Self(*b"ispe");
    /// Image rotation box `irot`.
    pub const IROT: Self = Self(*b"irot");
    /// Image mirror box `imir`.
    pub const IMIR: Self = Self(*b"imir");
    /// Colour information box `colr`.
    pub const COLR: Self = Self(*b"colr");
    /// Auxiliary item property `auxC`.
    pub const AUXC: Self = Self(*b"auxC");
    /// HEVC configuration box `hvcC`.
    pub const HVCC: Self = Self(*b"hvcC");
    /// Pixel aspect ratio box `pasp`.
    pub const PASP: Self = Self(*b"pasp");
    /// Clean aperture box `clap`.
    pub const CLAP: Self = Self(*b"clap");

    // Handlers & Item Types
    /// Picture handler type `pict`.
    pub const PICT: Self = Self(*b"pict");
    /// HEVC/H.265 intra item type `hvc1`.
    pub const HVC1: Self = Self(*b"hvc1");
    /// Image grid derived item type `grid`.
    pub const GRID: Self = Self(*b"grid");
    /// EXIF metadata item type `Exif`.
    pub const EXIF: Self = Self(*b"Exif");
    /// MIME metadata item type `mime`.
    pub const MIME: Self = Self(*b"mime");

    // Reference Types
    /// Derived image reference `dimg`.
    pub const DIMG: Self = Self(*b"dimg");
    /// Auxiliary image reference `auxl`.
    pub const AUXL: Self = Self(*b"auxl");
    /// Content description reference `cdsc`.
    pub const CDSC: Self = Self(*b"cdsc");
    /// Thumbnail reference `thmb`.
    pub const THMB: Self = Self(*b"thmb");
}

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = String::from_utf8_lossy(&self.0);
        write!(f, "{s}")
    }
}

/// Generic ISOBMFF Box header (8-byte standard or 16-byte extended).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoxHeader {
    /// The four-character code of this box.
    pub box_type: FourCC,
    /// Total box size in bytes including header.
    pub size: u64,
    /// Header size in bytes (8 for standard 32-bit, 16 for extended 64-bit).
    pub header_size: usize,
}

impl BoxHeader {
    /// Parses a box header from the beginning of a byte slice.
    pub fn parse(input: &[u8]) -> HeicResult<Self> {
        if input.len() < 8 {
            return Err(HeicError::MalformedInput(
                "Insufficient bytes for ISOBMFF box header (need >= 8 bytes)".to_string(),
            ));
        }

        let size_raw = u32::from_be_bytes([input[0], input[1], input[2], input[3]]);
        let box_type = FourCC([input[4], input[5], input[6], input[7]]);

        if size_raw == 1 {
            // 64-bit extended size
            if input.len() < 16 {
                return Err(HeicError::MalformedInput(
                    "Insufficient bytes for extended 64-bit box header (need >= 16 bytes)"
                        .to_string(),
                ));
            }
            let extended_size = u64::from_be_bytes([
                input[8], input[9], input[10], input[11], input[12], input[13], input[14],
                input[15],
            ]);
            if extended_size < 16 {
                return Err(HeicError::InvalidContainer(format!(
                    "Invalid 64-bit box size {extended_size} < header size 16 for {box_type}"
                )));
            }
            Ok(Self {
                box_type,
                size: extended_size,
                header_size: 16,
            })
        } else if size_raw == 0 {
            // Box extends to end of input
            Ok(Self {
                box_type,
                size: input.len() as u64,
                header_size: 8,
            })
        } else {
            if (size_raw as usize) < 8 {
                return Err(HeicError::InvalidContainer(format!(
                    "Invalid box size {size_raw} < header size 8 for {box_type}"
                )));
            }
            Ok(Self {
                box_type,
                size: size_raw as u64,
                header_size: 8,
            })
        }
    }

    /// Computes payload size in bytes excluding header bytes.
    #[inline]
    pub const fn payload_size(&self) -> u64 {
        self.size.saturating_sub(self.header_size as u64)
    }

    /// Extracts the payload slice corresponding to this box from `input`.
    pub fn payload<'a>(&self, input: &'a [u8]) -> HeicResult<&'a [u8]> {
        let total_size = usize::try_from(self.size).map_err(|_| {
            HeicError::LimitExceeded("Box size exceeds addressable memory".to_string())
        })?;
        if input.len() < total_size {
            return Err(HeicError::MalformedInput(format!(
                "Box {} truncated: expected {} bytes, got {}",
                self.box_type,
                total_size,
                input.len()
            )));
        }
        Ok(&input[self.header_size..total_size])
    }
}

/// Full Box Header containing version and flags (ISO/IEC 14496-12 Section 4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FullBoxHeader {
    /// Base box header.
    pub header: BoxHeader,
    /// Box specification version (typically 0 or 1).
    pub version: u8,
    /// 24-bit integer flags.
    pub flags: u32,
    /// Total header length including 4 bytes of version/flags.
    pub total_header_size: usize,
}

impl FullBoxHeader {
    /// Parses a Full Box Header from input.
    pub fn parse(input: &[u8]) -> HeicResult<Self> {
        let header = BoxHeader::parse(input)?;
        let start = header.header_size;
        if input.len() < start + 4 {
            return Err(HeicError::MalformedInput(format!(
                "Insufficient bytes for FullBox header in {}",
                header.box_type
            )));
        }

        let version = input[start];
        let flags = ((input[start + 1] as u32) << 16)
            | ((input[start + 2] as u32) << 8)
            | (input[start + 3] as u32);

        Ok(Self {
            header,
            version,
            flags,
            total_header_size: start + 4,
        })
    }

    /// Extracts the payload slice excluding full box version/flags.
    pub fn payload<'a>(&self, input: &'a [u8]) -> HeicResult<&'a [u8]> {
        let total_size = usize::try_from(self.header.size).map_err(|_| {
            HeicError::LimitExceeded("Box size exceeds addressable memory".to_string())
        })?;
        if input.len() < total_size {
            return Err(HeicError::MalformedInput(format!(
                "Full box {} truncated: expected {} bytes, got {}",
                self.header.box_type,
                total_size,
                input.len()
            )));
        }
        Ok(&input[self.total_header_size..total_size])
    }
}

/// Iterator over sibling boxes in a container payload.
pub struct BoxIter<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> BoxIter<'a> {
    /// Creates a new box iterator over a byte slice.
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }
}

impl<'a> Iterator for BoxIter<'a> {
    type Item = HeicResult<(BoxHeader, &'a [u8])>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.data.len() {
            return None;
        }

        let remaining = &self.data[self.offset..];
        if remaining.len() < 8 {
            self.offset = self.data.len();
            return Some(Err(HeicError::MalformedInput(
                "Trailing bytes insufficient for box header".to_string(),
            )));
        }

        let header = match BoxHeader::parse(remaining) {
            Ok(h) => h,
            Err(e) => {
                self.offset = self.data.len();
                return Some(Err(e));
            }
        };

        let box_size = match usize::try_from(header.size) {
            Ok(s) => s,
            Err(_) => {
                self.offset = self.data.len();
                return Some(Err(HeicError::LimitExceeded(
                    "Box size exceeds usize".to_string(),
                )));
            }
        };

        if remaining.len() < box_size {
            self.offset = self.data.len();
            return Some(Err(HeicError::MalformedInput(format!(
                "Box {} declares {} bytes but only {} remain",
                header.box_type,
                box_size,
                remaining.len()
            ))));
        }

        let box_data = &remaining[..box_size];
        self.offset += box_size;
        Some(Ok((header, box_data)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
