//! Major brand and compatible brands variation fixture builders.

use super::box_builder::*;
use super::hevc_builder::*;

/// Builds a HEIC container with major brand `mif1` and compatible brands `[mif1, heic, heix]`.
pub fn build_mif1_heic_brand() -> Vec<u8> {
    let ftyp = make_ftyp(b"mif1", 0, &[b"mif1", b"heic", b"heix"]);

    let hdlr = make_hdlr();
    let pitm = make_pitm(1);
    let infe = make_infe(1, b"hvc1");
    let iinf = make_iinf(&[infe]);

    let ispe = make_ispe(800, 600);
    let iprp = make_iprp(&[ispe], &[(1, &[1])]);

    let mdat_payload = mock_hevc_annex_b(16);

    let iloc_stub = make_iloc(&[(1, 0, mdat_payload.len() as u32)]);
    let mut meta_content = Vec::new();
    meta_content.extend_from_slice(&hdlr);
    meta_content.extend_from_slice(&pitm);
    meta_content.extend_from_slice(&iinf);
    meta_content.extend_from_slice(&iprp);
    meta_content.extend_from_slice(&iloc_stub);

    let meta_total_size = 12 + meta_content.len();
    let mdat_offset = (ftyp.len() + meta_total_size + 8) as u32;

    let iloc = make_iloc(&[(1, mdat_offset, mdat_payload.len() as u32)]);

    let mut meta_final = Vec::new();
    meta_final.extend_from_slice(&hdlr);
    meta_final.extend_from_slice(&pitm);
    meta_final.extend_from_slice(&iinf);
    meta_final.extend_from_slice(&iprp);
    meta_final.extend_from_slice(&iloc);
    let meta = make_full_box(b"meta", 0, 0, &meta_final);

    let mdat = make_box(b"mdat", &mdat_payload);

    let mut file = Vec::new();
    file.extend_from_slice(&ftyp);
    file.extend_from_slice(&meta);
    file.extend_from_slice(&mdat);
    file
}

/// Builds an unsupported ISOBMFF container (e.g. `mp42` / `isom` video file).
pub fn build_unsupported_mp4_container() -> Vec<u8> {
    let ftyp = make_ftyp(b"mp42", 0, &[b"mp42", b"isom"]);
    let moov = make_box(b"moov", &[0; 32]);
    let mut file = Vec::new();
    file.extend_from_slice(&ftyp);
    file.extend_from_slice(&moov);
    file
}
