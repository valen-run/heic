//! Integration tests for Display P3 and wide-gamut color profile containers.

use crate::fixtures::build_display_p3_heic;
use valen_heic_core::{ColorSpace, Limits};
use valen_heif_parser::{inspect_container, parse_heif};

#[test]
fn test_display_p3_container_inspection() {
    let data = build_display_p3_heic();
    let limits = Limits::default();
    let meta = inspect_container(&data, &limits).expect("Inspection should succeed");

    assert_eq!(meta.color_space, ColorSpace::DisplayP3);
    assert_eq!(meta.dimensions.width, 3840);
    assert_eq!(meta.dimensions.height, 2160);
}

#[test]
fn test_display_p3_heif_parsing() {
    let data = build_display_p3_heic();
    let limits = Limits::default();
    let heif = parse_heif(&data, &limits).expect("Parse should succeed");

    let primary = heif.items.get(&heif.primary_item_id).unwrap();
    assert_eq!(primary.color_space, ColorSpace::DisplayP3);
}
