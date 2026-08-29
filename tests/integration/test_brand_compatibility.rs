//! Integration tests for brand compatibility variations (`mif1`, `heic`, `heix`, etc.).

use crate::fixtures::{build_mif1_heic_brand, build_unsupported_mp4_container};
use valen_heic_core::{HeicError, Limits};
use valen_heif_parser::{inspect_container, is_heif_or_heic, parse_heif};

#[test]
fn test_mif1_brand_compatibility() {
    let data = build_mif1_heic_brand();
    assert!(is_heif_or_heic(&data));

    let limits = Limits::default();
    let meta = inspect_container(&data, &limits).expect("Inspection should succeed");
    assert_eq!(&meta.major_brand, b"mif1");
    assert!(meta.compatible_brands.contains(b"heic"));
}

#[test]
fn test_unsupported_mp4_rejection() {
    let data = build_unsupported_mp4_container();
    assert!(!is_heif_or_heic(&data));

    let limits = Limits::default();
    let err = inspect_container(&data, &limits).unwrap_err();
    assert!(matches!(
        err,
        HeicError::UnsupportedFormat(_) | HeicError::UnsupportedBrand(_)
    ));

    let err2 = parse_heif(&data, &limits).unwrap_err();
    assert!(matches!(
        err2,
        HeicError::UnsupportedFormat(_) | HeicError::UnsupportedBrand(_)
    ));
}
