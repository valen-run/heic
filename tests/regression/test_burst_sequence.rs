//! Regression tests for burst sequence containers (`.heifs` / `msf1`).

use crate::fixtures::build_burst_sequence_heifs;
use valen_heic_core::Limits;
use valen_heif_parser::{inspect_container, is_heif_or_heic, parse_heif};

#[test]
fn test_burst_sequence_detection() {
    let data = build_burst_sequence_heifs();
    // msf1 is a supported brand
    assert!(is_heif_or_heic(&data));
}

#[test]
fn test_burst_sequence_parsing_and_item_count() {
    let data = build_burst_sequence_heifs();
    let limits = Limits::default();
    let meta = inspect_container(&data, &limits).expect("Burst sequence inspection should succeed");

    assert_eq!(&meta.major_brand, b"msf1");
    assert_eq!(meta.image_count, 4);
    assert_eq!(meta.dimensions.width, 1920);
    assert_eq!(meta.dimensions.height, 1080);
}

#[test]
fn test_burst_sequence_heif_file() {
    let data = build_burst_sequence_heifs();
    let limits = Limits::default();
    let heif = parse_heif(&data, &limits).expect("Burst sequence parse should succeed");

    assert_eq!(heif.items.len(), 4);
    assert_eq!(heif.primary_item_id, 1);
}
