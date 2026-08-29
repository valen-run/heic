//! 10-bit wide-gamut Display P3 HEIC fixture builder.

use super::box_builder::*;
use super::hevc_builder::*;

/// Builds a synthetic Display P3 10-bit HEIC file container with `colr` NCLX box.
pub fn build_display_p3_heic() -> Vec<u8> {
    let ftyp = make_ftyp(b"heic", 0, &[b"heic", b"mif1", b"heix"]);

    let hdlr = make_hdlr();
    let pitm = make_pitm(1);
    let infe = make_infe(1, b"hvc1");
    let iinf = make_iinf(&[infe]);

    let ispe = make_ispe(3840, 2160);
    // Display P3 NCLX: colour_primaries=12 (P3), transfer=13, matrix=6, full_range=true
    let colr = make_colr_nclx(12, 13, 6, true);
    let iprp = make_iprp(&[ispe, colr], &[(1, &[1, 2])]);

    let mdat_payload = mock_hevc_annex_b(64);

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
