//! Integration tests for portrait mode with auxiliary alpha mask.

use crate::fixtures::build_portrait_alpha_heic;
use valen_heic_core::Limits;
use valen_heif_parser::{inspect_container, parse_heif};

#[test]
fn test_portrait_alpha_container_inspection() {
    let data = build_portrait_alpha_heic();
    let limits = Limits::default();
    let meta = inspect_container(&data, &limits).expect("Inspection should succeed");

    assert!(meta.has_alpha);
    assert_eq!(meta.alpha_item_id, Some(2));
    assert_eq!(meta.dimensions.width, 1200);
    assert_eq!(meta.dimensions.height, 1600);
}

#[test]
fn test_portrait_alpha_item_resolution() {
    let data = build_portrait_alpha_heic();
    let limits = Limits::default();
    let heif = parse_heif(&data, &limits).expect("Parse should succeed");

    assert_eq!(heif.alpha_item_id, Some(2));
    let alpha_item = heif.items.get(&2).expect("Alpha item should exist");
    assert!(alpha_item.is_alpha_mask);
}
