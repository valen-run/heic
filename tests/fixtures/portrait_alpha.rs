//! Portrait mode with auxiliary alpha channel fixture builder.

use super::box_builder::*;
use super::hevc_builder::*;

/// Builds a synthetic HEIC container with primary image (item 1) and auxiliary alpha plane (item 2).
pub fn build_portrait_alpha_heic() -> Vec<u8> {
    let ftyp = make_ftyp(b"heic", 0, &[b"heic", b"mif1"]);

    let hdlr = make_hdlr();
    let pitm = make_pitm(1);

    let infe1 = make_infe(1, b"hvc1"); // Primary color image
    let infe2 = make_infe(2, b"hvc1"); // Auxiliary alpha plane
    let iinf = make_iinf(&[infe1, infe2]);

    // iref: item 1 has auxiliary item 2 via 'auxl'
    let iref = make_iref(b"auxl", 1, &[2]);

    let ispe = make_ispe(1200, 1600);
    let auxc = make_auxc("urn:mpeg:hevc:2015:auxid:1");
    let iprp = make_iprp(&[ispe, auxc], &[(1, &[1]), (2, &[1, 2])]);

    let primary_bytes = mock_hevc_annex_b(32);
    let alpha_bytes = mock_hevc_annex_b(32);

    let len1 = primary_bytes.len() as u32;
    let len2 = alpha_bytes.len() as u32;

    let iloc_stub = make_iloc(&[(1, 0, len1), (2, 0, len2)]);
    let mut meta_content = Vec::new();
    meta_content.extend_from_slice(&hdlr);
    meta_content.extend_from_slice(&pitm);
    meta_content.extend_from_slice(&iinf);
    meta_content.extend_from_slice(&iref);
    meta_content.extend_from_slice(&iprp);
    meta_content.extend_from_slice(&iloc_stub);

    let meta_total_size = 12 + meta_content.len();
    let mdat_offset = (ftyp.len() + meta_total_size + 8) as u32;

    let iloc = make_iloc(&[(1, mdat_offset, len1), (2, mdat_offset + len1, len2)]);

    let mut meta_final = Vec::new();
    meta_final.extend_from_slice(&hdlr);
    meta_final.extend_from_slice(&pitm);
    meta_final.extend_from_slice(&iinf);
    meta_final.extend_from_slice(&iref);
    meta_final.extend_from_slice(&iprp);
    meta_final.extend_from_slice(&iloc);
    let meta = make_full_box(b"meta", 0, 0, &meta_final);

    let mut mdat_payload = Vec::new();
    mdat_payload.extend_from_slice(&primary_bytes);
    mdat_payload.extend_from_slice(&alpha_bytes);
    let mdat = make_box(b"mdat", &mdat_payload);

    let mut file = Vec::new();
    file.extend_from_slice(&ftyp);
    file.extend_from_slice(&meta);
    file.extend_from_slice(&mdat);
    file
}
