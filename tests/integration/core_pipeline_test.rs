//! End-to-end integration tests for HEIF container inspection and limit enforcement.

use valen_heic_core::{HeicError, Limits};
use valen_heif_parser::{inspect_container, is_heif_or_heic};

fn make_box(fourcc: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let size = (8 + payload.len()) as u32;
    let mut b = Vec::with_capacity(size as usize);
    b.extend_from_slice(&size.to_be_bytes());
    b.extend_from_slice(fourcc);
    b.extend_from_slice(payload);
    b
}

fn make_full_box(fourcc: &[u8; 4], version: u8, flags: u32, payload: &[u8]) -> Vec<u8> {
    let size = (12 + payload.len()) as u32;
    let mut b = Vec::with_capacity(size as usize);
    b.extend_from_slice(&size.to_be_bytes());
    b.extend_from_slice(fourcc);
    b.push(version);
    b.push(((flags >> 16) & 0xFF) as u8);
    b.push(((flags >> 8) & 0xFF) as u8);
    b.push((flags & 0xFF) as u8);
    b.extend_from_slice(payload);
    b
}

fn create_synthetic_heic_file() -> Vec<u8> {
    // 1. ftyp
    let mut ftyp_payload = Vec::new();
    ftyp_payload.extend_from_slice(b"heic");
    ftyp_payload.extend_from_slice(&[0, 0, 0, 0]);
    ftyp_payload.extend_from_slice(b"mif1");
    let ftyp = make_box(b"ftyp", &ftyp_payload);

    // 2. meta
    let hdlr = make_full_box(
        b"hdlr",
        0,
        0,
        &[
            0, 0, 0, 0, b'p', b'i', b'c', b't', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
    );
    let pitm = make_full_box(b"pitm", 0, 0, &1u16.to_be_bytes());
    let infe = make_full_box(b"infe", 2, 0, &[0, 1, 0, 0, b'h', b'v', b'c', b'1', 0]);
    let mut iinf_payload = Vec::new();
    iinf_payload.extend_from_slice(&1u16.to_be_bytes());
    iinf_payload.extend_from_slice(&infe);
    let iinf = make_full_box(b"iinf", 0, 0, &iinf_payload);

    // ispe
    let mut ispe_payload = Vec::new();
    ispe_payload.extend_from_slice(&800u32.to_be_bytes());
    ispe_payload.extend_from_slice(&600u32.to_be_bytes());
    let ispe = make_full_box(b"ispe", 0, 0, &ispe_payload);
    let ipco = make_box(b"ipco", &ispe);

    let mut ipma_payload = Vec::new();
    ipma_payload.extend_from_slice(&1u32.to_be_bytes());
    ipma_payload.extend_from_slice(&1u16.to_be_bytes());
    ipma_payload.push(1);
    ipma_payload.push(1);
    let ipma = make_full_box(b"ipma", 0, 0, &ipma_payload);

    let mut iprp_payload = Vec::new();
    iprp_payload.extend_from_slice(&ipco);
    iprp_payload.extend_from_slice(&ipma);
    let iprp = make_box(b"iprp", &iprp_payload);

    let mut iloc_payload = Vec::new();
    iloc_payload.push(0x44);
    iloc_payload.push(0x00);
    iloc_payload.extend_from_slice(&1u16.to_be_bytes());
    iloc_payload.extend_from_slice(&1u16.to_be_bytes());
    iloc_payload.extend_from_slice(&0u16.to_be_bytes());
    iloc_payload.extend_from_slice(&1u16.to_be_bytes());
    iloc_payload.extend_from_slice(&0u32.to_be_bytes());
    iloc_payload.extend_from_slice(&0u32.to_be_bytes());
    let iloc = make_full_box(b"iloc", 0, 0, &iloc_payload);

    let mut meta_content = Vec::new();
    meta_content.extend_from_slice(&hdlr);
    meta_content.extend_from_slice(&pitm);
    meta_content.extend_from_slice(&iinf);
    meta_content.extend_from_slice(&iprp);
    meta_content.extend_from_slice(&iloc);
    let meta = make_full_box(b"meta", 0, 0, &meta_content);

    let mut file_bytes = Vec::new();
    file_bytes.extend_from_slice(&ftyp);
    file_bytes.extend_from_slice(&meta);
    file_bytes
}

#[test]
fn test_integration_detection_and_inspection() {
    let data = create_synthetic_heic_file();
    assert!(is_heif_or_heic(&data));

    let limits = Limits::none().with_max_file_size(1024 * 1024);
    let meta = inspect_container(&data, &limits).expect("Inspection should succeed");
    assert_eq!(&meta.major_brand, b"heic");
    assert_eq!(meta.dimensions.width, 800);
    assert_eq!(meta.dimensions.height, 600);
}

#[test]
fn test_integration_limits_rejection() {
    let data = create_synthetic_heic_file();
    let limits = Limits::none().with_max_file_size(10); // file size is > 10
    let err = inspect_container(&data, &limits).unwrap_err();
    assert!(matches!(
        err,
        HeicError::LimitInputBytes { .. } | HeicError::LimitExceeded(_)
    ));
}

#[test]
fn test_inspect_incomplete_container_returns_structured_error() {
    // Only ftyp box, missing meta box
    let mut ftyp_payload = Vec::new();
    ftyp_payload.extend_from_slice(b"heic");
    ftyp_payload.extend_from_slice(&[0, 0, 0, 0]);
    ftyp_payload.extend_from_slice(b"mif1");
    let ftyp_only = make_box(b"ftyp", &ftyp_payload);

    assert!(is_heif_or_heic(&ftyp_only));
    let limits = Limits::none();
    let err = inspect_container(&ftyp_only, &limits).unwrap_err();
    assert!(matches!(err, HeicError::InvalidContainer(_)));
}
