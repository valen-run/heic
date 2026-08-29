//! Burst sequence image container (`.heifs` / `msf1`) fixture builder.

use super::box_builder::*;
use super::hevc_builder::*;

/// Builds a burst sequence container with `msf1` major brand and multiple frame items.
pub fn build_burst_sequence_heifs() -> Vec<u8> {
    let ftyp = make_ftyp(b"msf1", 0, &[b"msf1", b"heis"]);

    let hdlr = make_hdlr();
    let pitm = make_pitm(1);

    // 4 burst frames
    let mut infe_boxes = Vec::new();
    for id in 1..=4 {
        infe_boxes.push(make_infe(id, b"hvc1"));
    }
    let iinf = make_iinf(&infe_boxes);

    let ispe = make_ispe(1920, 1080);
    let iprp = make_iprp(&[ispe], &[(1, &[1]), (2, &[1]), (3, &[1]), (4, &[1])]);

    let frame_payload = mock_hevc_annex_b(16);
    let frame_len = frame_payload.len() as u32;

    let iloc_stub = make_iloc(&[
        (1, 0, frame_len),
        (2, 0, frame_len),
        (3, 0, frame_len),
        (4, 0, frame_len),
    ]);

    let mut meta_content = Vec::new();
    meta_content.extend_from_slice(&hdlr);
    meta_content.extend_from_slice(&pitm);
    meta_content.extend_from_slice(&iinf);
    meta_content.extend_from_slice(&iprp);
    meta_content.extend_from_slice(&iloc_stub);

    let meta_total_size = 12 + meta_content.len();
    let mdat_offset = (ftyp.len() + meta_total_size + 8) as u32;

    let iloc = make_iloc(&[
        (1, mdat_offset, frame_len),
        (2, mdat_offset + frame_len, frame_len),
        (3, mdat_offset + frame_len * 2, frame_len),
        (4, mdat_offset + frame_len * 3, frame_len),
    ]);

    let mut meta_final = Vec::new();
    meta_final.extend_from_slice(&hdlr);
    meta_final.extend_from_slice(&pitm);
    meta_final.extend_from_slice(&iinf);
    meta_final.extend_from_slice(&iprp);
    meta_final.extend_from_slice(&iloc);
    let meta = make_full_box(b"meta", 0, 0, &meta_final);

    let mut mdat_payload = Vec::new();
    for _ in 1..=4 {
        mdat_payload.extend_from_slice(&frame_payload);
    }
    let mdat = make_box(b"mdat", &mdat_payload);

    let mut file = Vec::new();
    file.extend_from_slice(&ftyp);
    file.extend_from_slice(&meta);
    file.extend_from_slice(&mdat);
    file
}
