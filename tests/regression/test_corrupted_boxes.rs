//! Regression tests for corrupted, truncated, and malformed box structures.

use crate::fixtures::{
    build_out_of_bounds_iloc, build_oversized_box_claim, build_truncated_header,
};
use valen_heic_core::{HeicError, Limits};
use valen_heif_parser::{inspect_container, is_heif_or_heic, parse_heif};

#[test]
fn test_truncated_box_header_rejection() {
    let truncated = build_truncated_header();
    assert!(!is_heif_or_heic(&truncated));

    let limits = Limits::default();
    let err = inspect_container(&truncated, &limits).unwrap_err();
    assert!(matches!(
        err,
        HeicError::UnsupportedFormat(_) | HeicError::InvalidContainer(_)
    ));
}

#[test]
fn test_oversized_box_claim_rejection() {
    let bad_box = build_oversized_box_claim();
    assert!(!is_heif_or_heic(&bad_box));

    let limits = Limits::default();
    let err = inspect_container(&bad_box, &limits).unwrap_err();
    assert!(matches!(
        err,
        HeicError::UnsupportedFormat(_)
            | HeicError::InvalidContainer(_)
            | HeicError::MalformedInput(_)
    ));
}

#[test]
fn test_out_of_bounds_iloc_rejection() {
    let bad_iloc = build_out_of_bounds_iloc();
    let limits = Limits::default();
    let err = parse_heif(&bad_iloc, &limits).unwrap_err();
    assert!(matches!(
        err,
        HeicError::InvalidContainer(_) | HeicError::MalformedInput(_)
    ));
}
