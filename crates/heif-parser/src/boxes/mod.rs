//! ISOBMFF Box header structures and parsing primitives.

use valen_heic_core::{HeicError, HeicResult};

/// Four-character box type identifier (FourCC).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FourCC(pub [u8; 4]);

impl FourCC {
    /// Creates a FourCC from a 4-byte slice.
    pub const fn from_bytes(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }

    /// File type box `ftyp`.
    pub const FTYP: Self = Self(*b"ftyp");
    /// Meta box `meta`.
    pub const META: Self = Self(*b"meta");
    /// Primary item box `pitm`.
    pub const PITM: Self = Self(*b"pitm");
    /// Item location box `iloc`.
    pub const ILOC: Self = Self(*b"iloc");
    /// Item information box `iinf`.
    pub const IINF: Self = Self(*b"iinf");
    /// Item properties box `iprp`.
    pub const IPRP: Self = Self(*b"iprp");
    /// Image spatial extents box `ispe`.
    pub const ISPE: Self = Self(*b"ispe");
    /// Media data box `mdat`.
    pub const MDAT: Self = Self(*b"mdat");
}

/// Generic ISOBMFF Box header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoxHeader {
    /// The four-character code of this box.
    pub box_type: FourCC,
    /// Total box size including header.
    pub size: u64,
    /// Header size in bytes (8 for standard 32-bit, 16 for extended 64-bit).
    pub header_size: usize,
}

impl BoxHeader {
    /// Parses a box header from a byte slice.
    pub fn parse(input: &[u8]) -> HeicResult<Self> {
        if input.len() < 8 {
            return Err(HeicError::MalformedInput(
                "Insufficient bytes for ISOBMFF box header".to_string(),
            ));
        }

        let size_raw = u32::from_be_bytes([input[0], input[1], input[2], input[3]]);
        let box_type = FourCC([input[4], input[5], input[6], input[7]]);

        if size_raw == 1 {
            // 64-bit extended size
            if input.len() < 16 {
                return Err(HeicError::MalformedInput(
                    "Insufficient bytes for extended 64-bit box header".to_string(),
                ));
            }
            let extended_size = u64::from_be_bytes([
                input[8], input[9], input[10], input[11], input[12], input[13], input[14],
                input[15],
            ]);
            Ok(Self {
                box_type,
                size: extended_size,
                header_size: 16,
            })
        } else if size_raw == 0 {
            // Box extends to end of file
            Ok(Self {
                box_type,
                size: input.len() as u64,
                header_size: 8,
            })
        } else {
            Ok(Self {
                box_type,
                size: size_raw as u64,
                header_size: 8,
            })
        }
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
    }
}
