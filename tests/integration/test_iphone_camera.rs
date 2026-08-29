//! Integration tests for 12MP iPhone camera HEIC containers.

use crate::fixtures::build_iphone_camera_12mp_heic;
use valen_heic_core::Limits;
use valen_heif_parser::{inspect_container, is_heif_or_heic, parse_heif};

#[test]
fn test_iphone_12mp_brand_detection() {
    let data = build_iphone_camera_12mp_heic();
    assert!(is_heif_or_heic(&data));
}

#[test]
fn test_iphone_12mp_container_inspection() {
    let data = build_iphone_camera_12mp_heic();
    let limits = Limits::default();
    let meta = inspect_container(&data, &limits).expect("Inspection should succeed");

    assert_eq!(&meta.major_brand, b"heic");
    assert_eq!(meta.dimensions.width, 4032);
    assert_eq!(meta.dimensions.height, 3024);
    assert!(!meta.is_grid);
    assert!(!meta.has_alpha);
}

#[test]
fn test_iphone_12mp_bitstream_extraction() {
    let data = build_iphone_camera_12mp_heic();
    let limits = Limits::default();
    let heif = parse_heif(&data, &limits).expect("Parse should succeed");

    let primary_bytes = heif
        .extract_item_data(&data, heif.primary_item_id)
        .expect("Primary data extraction should succeed");

    assert!(!primary_bytes.is_empty());
}
