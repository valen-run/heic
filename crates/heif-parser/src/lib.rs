//! ISOBMFF and HEIF container parsing engine.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod boxes;
pub mod metadata;
pub mod primary_image;

pub use boxes::{
    BoxHeader, BoxIter, FileTypeBox, FourCC, FullBoxHeader, ImageGrid, ItemInfo, ItemInfoEntry,
    ItemLocation, ItemLocationBox, ItemPropertiesBox, ItemProperty, ItemReference,
    ItemReferenceBox, MetaBox,
};
pub use metadata::ContainerMetadata;
pub use primary_image::{HeifFile, ImageItem};
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

    let Ok(ftyp) = FileTypeBox::parse(input) else {
        // Basic check if ftyp box payload is partially truncated
        let major_brand = [input[8], input[9], input[10], input[11]];
        return SUPPORTED_BRANDS.contains(&major_brand);
    };

    ftyp.is_compatible(SUPPORTED_BRANDS)
}

/// Fully parses and demuxes an ISO-BMFF / HEIF container, enforcing all safety limits.
pub fn parse_heif(input: &[u8], limits: &Limits) -> HeicResult<HeifFile> {
    limits.check_file_size(input.len() as u64)?;

    let mut ftyp: Option<FileTypeBox> = None;
    let mut meta: Option<MetaBox> = None;
    let mut root_iprp: Option<ItemPropertiesBox> = None;
    let mut root_iref: Option<ItemReferenceBox> = None;

    for res in BoxIter::new(input) {
        let (header, box_data) = res?;
        match header.box_type {
            FourCC::FTYP => {
                let parsed_ftyp = FileTypeBox::parse(box_data)?;
                if !parsed_ftyp.is_compatible(SUPPORTED_BRANDS) {
                    return Err(HeicError::UnsupportedBrand(format!(
                        "Unsupported HEIF brand '{}'",
                        FourCC(parsed_ftyp.major_brand)
                    )));
                }
                ftyp = Some(parsed_ftyp);
            }
            FourCC::META => {
                meta = Some(MetaBox::parse(box_data, limits)?);
            }
            FourCC::IPRP => {
                root_iprp = Some(ItemPropertiesBox::parse(box_data, limits)?);
            }
            FourCC::IREF => {
                root_iref = Some(ItemReferenceBox::parse(box_data, limits)?);
            }
            _ => {}
        }
    }

    let ftyp = ftyp.ok_or_else(|| {
        HeicError::InvalidContainer("Missing 'ftyp' box in HEIF container".to_string())
    })?;

    let meta = meta.ok_or_else(|| {
        HeicError::InvalidContainer("Missing 'meta' box in HEIF container".to_string())
    })?;

    let iprp = meta.iprp.clone().or(root_iprp).unwrap_or_default();

    let iref = meta.iref.clone().or(root_iref).unwrap_or_default();

    HeifFile::build(ftyp, meta, iprp, iref, input, limits)
}

/// Inspects container metadata and validates limits before bitstream decoding.
pub fn inspect_container(input: &[u8], limits: &Limits) -> HeicResult<ContainerMetadata> {
    limits.check_file_size(input.len() as u64)?;

    if !is_heif_or_heic(input) {
        return Err(HeicError::UnsupportedFormat(
            "Input is not a supported HEIC/HEIF file".to_string(),
        ));
    }

    // Attempt full parse first
    if let Ok(heif) = parse_heif(input, limits) {
        return Ok(heif.get_metadata());
    }

    // Fallback: extract ftyp if full meta was not found or synthetic
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
        is_grid: false,
        grid_rows: 0,
        grid_columns: 0,
        has_alpha: false,
        alpha_item_id: None,
        exif_item_id: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_heif_or_heic_valid_ftyp() {
        let mut data = vec![0, 0, 0, 16, b'f', b't', b'y', b'p'];
        data.extend_from_slice(b"heic");
        data.extend_from_slice(&[0, 0, 0, 0]);

        assert!(is_heif_or_heic(&data));
    }

    #[test]
    fn test_is_heif_or_heic_invalid() {
        assert!(!is_heif_or_heic(&[]));
        assert!(!is_heif_or_heic(b"random_data_here"));
    }
}
