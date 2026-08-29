//! Corrupted, truncated, and malformed HEIC container fixture builders.

use super::box_builder::*;

/// Builds a container with a truncated box header (< 8 bytes).
pub fn build_truncated_header() -> Vec<u8> {
    vec![0x00, 0x00, 0x00, 0x18, b'f', b't']
}

/// Builds a container with an impossible 32-bit box size (e.g. 2GB size claim on a 20-byte buffer).
pub fn build_oversized_box_claim() -> Vec<u8> {
    let mut file = Vec::new();
    file.extend_from_slice(&0x7FFFFFFF_u32.to_be_bytes()); // 2GB box claim
    file.extend_from_slice(b"ftyp");
    file.extend_from_slice(b"heic\0\0\0\0");
    file
}

/// Builds an `iloc` box that references offsets beyond the end of the file.
pub fn build_out_of_bounds_iloc() -> Vec<u8> {
    let ftyp = make_ftyp(b"heic", 0, &[b"heic"]);
    let hdlr = make_hdlr();
    let pitm = make_pitm(1);
    let infe = make_infe(1, b"hvc1");
    let iinf = make_iinf(&[infe]);
    let ispe = make_ispe(100, 100);
    let iprp = make_iprp(&[ispe], &[(1, &[1])]);

    // Offset 999999 is well beyond file bounds
    let iloc = make_iloc(&[(1, 999_999, 1000)]);

    let mut meta_content = Vec::new();
    meta_content.extend_from_slice(&hdlr);
    meta_content.extend_from_slice(&pitm);
    meta_content.extend_from_slice(&iinf);
    meta_content.extend_from_slice(&iprp);
    meta_content.extend_from_slice(&iloc);
    let meta = make_full_box(b"meta", 0, 0, &meta_content);

    let mut file = Vec::new();
    file.extend_from_slice(&ftyp);
    file.extend_from_slice(&meta);
    file
}

/// Builds an incomplete HEVC NAL slice payload (truncated in the middle of CABAC header).
pub fn build_truncated_slice_payload() -> Vec<u8> {
    vec![0x00, 0x00, 0x00, 0x01, 0x26, 0x01] // Only 2 bytes of IDR slice
}
