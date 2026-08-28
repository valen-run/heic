//! End-to-end integration tests for HEIF container inspection and limit enforcement.

use valen_heic_core::{HeicError, Limits};
use valen_heif_parser::{inspect_container, is_heif_or_heic};

fn create_synthetic_heic_header() -> Vec<u8> {
    let mut header = vec![0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p'];
    header.extend_from_slice(b"heic"); // major brand
    header.extend_from_slice(&[0, 0, 0, 0]); // minor version
    header.extend_from_slice(b"mif1"); // compatible brand
    header
}

#[test]
fn test_integration_detection_and_inspection() {
    let data = create_synthetic_heic_header();
    assert!(is_heif_or_heic(&data));

    let limits = Limits::none().with_max_file_size(1024);
    let meta = inspect_container(&data, &limits).expect("Inspection should succeed");
    assert_eq!(&meta.major_brand, b"heic");
}

#[test]
fn test_integration_limits_rejection() {
    let data = create_synthetic_heic_header();
    let limits = Limits::none().with_max_file_size(10); // file size is 24 > 10
    let err = inspect_container(&data, &limits).unwrap_err();
    assert!(matches!(err, HeicError::LimitExceeded(_)));
}
