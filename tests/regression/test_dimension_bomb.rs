//! Regression tests for 65k dimension bomb DoS protection.

use crate::fixtures::build_65k_dimension_bomb_heic;
use valen_heic_core::{HeicError, Limits};
use valen_heif_parser::{inspect_container, parse_heif};

#[test]
fn test_65k_dimension_bomb_rejection_on_max_width() {
    let bomb_data = build_65k_dimension_bomb_heic();
    // Browser defaults restrict to 16,384 max width/height
    let limits = Limits::default_browser();

    let err = inspect_container(&bomb_data, &limits).unwrap_err();
    assert!(matches!(
        err,
        HeicError::LimitDimensions { .. }
            | HeicError::LimitPixels { .. }
            | HeicError::LimitExceeded(_)
    ));
}

#[test]
fn test_65k_dimension_bomb_rejection_on_parse() {
    let bomb_data = build_65k_dimension_bomb_heic();
    let limits = Limits::none().with_max_width(8192).with_max_height(8192);

    let err = parse_heif(&bomb_data, &limits).unwrap_err();
    assert!(matches!(
        err,
        HeicError::LimitDimensions { .. }
            | HeicError::LimitPixels { .. }
            | HeicError::LimitExceeded(_)
    ));
}

#[test]
fn test_65k_dimension_bomb_pixel_count_limit() {
    let bomb_data = build_65k_dimension_bomb_heic();
    let limits = Limits::none().with_max_pixel_count(10_000_000); // 10 MP limit

    let err = inspect_container(&bomb_data, &limits).unwrap_err();
    assert!(matches!(
        err,
        HeicError::LimitPixels { .. }
            | HeicError::LimitDimensions { .. }
            | HeicError::LimitExceeded(_)
    ));
}
