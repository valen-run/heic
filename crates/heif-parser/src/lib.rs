//! ISOBMFF and HEIF container parsing engine.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod boxes;
pub mod metadata;
pub mod primary_image;

pub use boxes::{BoxHeader, FourCC};
pub use metadata::ContainerMetadata;
pub use primary_image::ImageItem;
use valen_heic_core::{HeicError, HeicResult, Limits};

/// Supported HEIF/HEIC brand identifiers.
pub const SUPPORTED_BRANDS: &[[u8; 4]] = &[
    *b"heic", *b"heix", *b"hevc", *b"hevx", *b"mif1", *b"msf1", *b"heis", *b"avic",
];

/// Quickly detects whether the input byte stream is a valid HEIF/HEIC container.
pub fn is_heif_or_heic(input: &[u8]) -> bool {
    if input.len() < 12 {
        return false;
    }

    let Ok(header) = BoxHeader::parse(input) else {
        return false;
    };

    if header.box_type != FourCC::FTYP {
        return false;
    }

    // Check major brand
    let major_brand = [input[8], input[9], input[10], input[11]];
    if SUPPORTED_BRANDS.contains(&major_brand) {
        return true;
    }

    // Check compatible brands
    let mut offset = 16;
    let end = (header.size as usize).min(input.len());
    while offset + 4 <= end {
        let brand = [
            input[offset],
            input[offset + 1],
            input[offset + 2],
            input[offset + 3],
        ];
        if SUPPORTED_BRANDS.contains(&brand) {
            return true;
        }
        offset += 4;
    }

    false
}

/// Inspects container metadata and validates limits before bitstream decoding.
pub fn inspect_container(input: &[u8], limits: &Limits) -> HeicResult<ContainerMetadata> {
    limits.check_file_size(input.len() as u64)?;

    if !is_heif_or_heic(input) {
        return Err(HeicError::UnsupportedFormat(
            "Input is not a supported HEIC/HEIF file".to_string(),
        ));
    }

    // Basic ftyp metadata extraction
    let header = BoxHeader::parse(input)?;
    let major_brand = [input[8], input[9], input[10], input[11]];
    let mut compatible_brands = Vec::new();

    let mut offset = 16;
    let end = (header.size as usize).min(input.len());
    while offset + 4 <= end {
        compatible_brands.push([
            input[offset],
            input[offset + 1],
            input[offset + 2],
            input[offset + 3],
        ]);
        offset += 4;
    }

    Ok(ContainerMetadata {
        major_brand,
        compatible_brands,
        dimensions: valen_heic_core::ImageDimensions::new(0, 0),
        color_space: valen_heic_core::ColorSpace::Srgb,
        orientation: None,
        primary_item_id: None,
        image_count: 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_heif_or_heic_invalid() {
        assert!(!is_heif_or_heic(b"not a valid heic"));
    }

    #[test]
    fn test_is_heif_or_heic_valid_ftyp() {
        let mut sample = vec![0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p'];
        sample.extend_from_slice(b"heic"); // major brand
        sample.extend_from_slice(&[0, 0, 0, 0]); // minor version
        sample.extend_from_slice(b"mif1"); // compatible brand

        assert!(is_heif_or_heic(&sample));
    }
}
