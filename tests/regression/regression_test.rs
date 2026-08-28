//! Regression tests suite for edge cases, truncated files, and reported issues.

use valen_heic_core::{HeicError, Limits};
use valen_heif_parser::{inspect_container, is_heif_or_heic};

#[test]
fn test_regression_empty_buffer() {
    let empty: &[u8] = &[];
    assert!(!is_heif_or_heic(empty));

    let limits = Limits::none();
    let err = inspect_container(empty, &limits).unwrap_err();
    assert!(matches!(err, HeicError::UnsupportedFormat(_)));
}

#[test]
fn test_regression_truncated_box_header() {
    // 6 bytes - truncated ISOBMFF header
    let truncated = &[0x00, 0x00, 0x00, 0x18, b'f', b't'];
    assert!(!is_heif_or_heic(truncated));
}
