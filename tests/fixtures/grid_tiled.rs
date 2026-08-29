//! 4x4 Grid tiled HEIC image fixture builder.

use super::box_builder::*;

/// Builds a synthetic 4x4 Grid tiled HEIC file container (16 tiles, 512x512 each -> 2048x2048 canvas).
pub fn build_4x4_grid_heic() -> Vec<u8> {
    let ftyp = make_ftyp(b"heic", 0, &[b"heic", b"mif1", b"msf1"]);

    let hdlr = make_hdlr();
    let pitm = make_pitm(1); // Item 1 is the grid descriptor

    // Item 1 is 'grid', items 2..=17 are 'hvc1' tiles
    let mut infe_boxes = Vec::new();
    infe_boxes.push(make_infe(1, b"grid"));
    for id in 2..=17 {
        infe_boxes.push(make_infe(id, b"hvc1"));
    }
    let iinf = make_iinf(&infe_boxes);

    // iref: item 1 references tiles 2..=17
    let tile_ids: Vec<u16> = (2..=17).collect();
    let iref = make_iref(b"dimg", 1, &tile_ids);

    // ispe: 2048x2048 output dimensions for grid item
    let ispe = make_ispe(2048, 2048);
    let iprp = make_iprp(&[ispe], &[(1, &[1])]);

    // Grid descriptor payload in mdat (8 bytes: ver 0, flags 0, rows_minus1=3, cols_minus1=3, w=2048, h=2048)
    let mut grid_desc = Vec::new();
    grid_desc.extend_from_slice(&[0, 0, 3, 3]); // rows-1=3 (4 rows), cols-1=3 (4 cols)
    grid_desc.extend_from_slice(&2048u16.to_be_bytes());
    grid_desc.extend_from_slice(&2048u16.to_be_bytes());

    let tile_len = 16u32;
    let mut mdat_payload = Vec::new();
    mdat_payload.extend_from_slice(&grid_desc);
    for i in 0..16 {
        mdat_payload.extend_from_slice(&[0x10 + (i as u8); 16]);
    }

    // iloc computation
    let mut iloc_items = Vec::new();
    iloc_items.push((1u16, 0u32, 8u32)); // item 1
    for id in 2..=17 {
        iloc_items.push((id, 0u32, tile_len));
    }
    let iloc_stub = make_iloc(&iloc_items);

    let mut meta_content = Vec::new();
    meta_content.extend_from_slice(&hdlr);
    meta_content.extend_from_slice(&pitm);
    meta_content.extend_from_slice(&iinf);
    meta_content.extend_from_slice(&iref);
    meta_content.extend_from_slice(&iprp);
    meta_content.extend_from_slice(&iloc_stub);

    let meta_total_size = 12 + meta_content.len();
    let mdat_offset = (ftyp.len() + meta_total_size + 8) as u32;

    let mut final_iloc_items = Vec::new();
    final_iloc_items.push((1u16, mdat_offset, 8u32));
    for (idx, id) in (2..=17).enumerate() {
        let offset = mdat_offset + 8 + (idx as u32 * tile_len);
        final_iloc_items.push((id, offset, tile_len));
    }
    let iloc = make_iloc(&final_iloc_items);

    let mut meta_final = Vec::new();
    meta_final.extend_from_slice(&hdlr);
    meta_final.extend_from_slice(&pitm);
    meta_final.extend_from_slice(&iinf);
    meta_final.extend_from_slice(&iref);
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
