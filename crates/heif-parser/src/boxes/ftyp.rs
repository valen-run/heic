//! `ftyp` (File Type Box) parser and brand compatibility verification.

use super::{BoxHeader, FourCC};
use valen_heic_core::{HeicError, HeicResult};

/// Represents an ISO-BMFF File Type Box (`ftyp`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTypeBox {
    /// The primary brand identifier (e.g. `heic`, `heix`, `mif1`).
    pub major_brand: [u8; 4],
    /// Minor format version / revision number.
    pub minor_version: u32,
    /// List of compatible brand identifiers.
    pub compatible_brands: Vec<[u8; 4]>,
}

impl FileTypeBox {
    /// Parses an `ftyp` box from raw box bytes (including header).
    pub fn parse(input: &[u8]) -> HeicResult<Self> {
        let header = BoxHeader::parse(input)?;
        if header.box_type != FourCC::FTYP {
            return Err(HeicError::InvalidContainer(format!(
                "Expected 'ftyp' box, got '{}'",
                header.box_type
            )));
        }

        let payload = header.payload(input)?;
        if payload.len() < 8 {
            return Err(HeicError::MalformedInput(
                "Insufficient bytes for ftyp payload (minimum 8 bytes required)".to_string(),
            ));
        }

        let major_brand = [payload[0], payload[1], payload[2], payload[3]];
        let minor_version = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);

        let mut compatible_brands = Vec::new();
        let mut offset = 8;
        while offset + 4 <= payload.len() {
            compatible_brands.push([
                payload[offset],
                payload[offset + 1],
                payload[offset + 2],
                payload[offset + 3],
            ]);
            offset += 4;
        }

        Ok(Self {
            major_brand,
            minor_version,
            compatible_brands,
        })
    }

    /// Checks if either the major brand or any compatible brand is in `supported_brands`.
    pub fn is_compatible(&self, supported_brands: &[[u8; 4]]) -> bool {
        if supported_brands.contains(&self.major_brand) {
            return true;
        }
        for brand in &self.compatible_brands {
            if supported_brands.contains(brand) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_ftyp() {
        let mut raw = vec![0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p'];
        raw.extend_from_slice(b"heic"); // major brand
        raw.extend_from_slice(&[0, 0, 0, 0]); // minor version
        raw.extend_from_slice(b"mif1"); // compatible brand 1
        raw.extend_from_slice(b"hevc"); // compatible brand 2

        let ftyp = FileTypeBox::parse(&raw).unwrap();
        assert_eq!(&ftyp.major_brand, b"heic");
        assert_eq!(ftyp.minor_version, 0);
        assert_eq!(ftyp.compatible_brands.len(), 2);
        assert_eq!(&ftyp.compatible_brands[0], b"mif1");
        assert_eq!(&ftyp.compatible_brands[1], b"hevc");

        assert!(ftyp.is_compatible(&[*b"heic", *b"mif1"]));
        assert!(!ftyp.is_compatible(&[*b"avif"]));
    }

    #[test]
    fn test_parse_short_ftyp() {
        let raw = [
            0x00, 0x00, 0x00, 0x0c, b'f', b't', b'y', b'p', b'h', b'e', b'i', b'c',
        ];
        assert!(FileTypeBox::parse(&raw).is_err());
    }
}
