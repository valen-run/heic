//! Integration tests for EXIF orientation tags parsing and transform execution.

use crate::fixtures::build_oriented_heic;
use valen_heic_core::{Limits, Orientation};
use valen_heif_parser::inspect_container;

#[test]
fn test_orientation_tags_resolution() {
    let limits = Limits::default();

    // Tag 1 (0° CCW / Normal)
    let d0 = build_oriented_heic(0);
    let m0 = inspect_container(&d0, &limits).unwrap();
    assert_eq!(m0.orientation, Some(1));

    // Tag 8 (90° CCW / 270° CW)
    let d1 = build_oriented_heic(1);
    let m1 = inspect_container(&d1, &limits).unwrap();
    assert_eq!(m1.orientation, Some(8));

    // Tag 3 (180° CCW / 180° CW)
    let d2 = build_oriented_heic(2);
    let m2 = inspect_container(&d2, &limits).unwrap();
    assert_eq!(m2.orientation, Some(3));

    // Tag 6 (270° CCW / 90° CW)
    let d3 = build_oriented_heic(3);
    let m3 = inspect_container(&d3, &limits).unwrap();
    assert_eq!(m3.orientation, Some(6));
}

#[test]
fn test_orientation_enum_conversion() {
    assert_eq!(Orientation::from_u8(1).unwrap(), Orientation::Normal);
    assert_eq!(Orientation::from_u8(3).unwrap(), Orientation::Rotate180);
    assert_eq!(Orientation::from_u8(6).unwrap(), Orientation::Rotate90);
    assert_eq!(Orientation::from_u8(8).unwrap(), Orientation::Rotate270);
}
