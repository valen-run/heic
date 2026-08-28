//! Comprehensive synthetic HEIF pipeline integration tests.

use valen_heic_core::Limits;
use valen_heif_parser::{
    boxes::iprp::properties::HevcConfigProperty, inspect_container, is_heif_or_heic, parse_heif,
};

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

fn build_synthetic_heic() -> Vec<u8> {
    // 1. ftyp
    let mut ftyp_payload = Vec::new();
    ftyp_payload.extend_from_slice(b"heic"); // major brand
    ftyp_payload.extend_from_slice(&[0, 0, 0, 0]); // minor version
    ftyp_payload.extend_from_slice(b"mif1"); // compatible brand
    let ftyp = make_box(b"ftyp", &ftyp_payload);

    // 2. meta box components
    // hdlr
    let mut hdlr_payload = Vec::new();
    hdlr_payload.extend_from_slice(&[0; 4]); // pre_defined
    hdlr_payload.extend_from_slice(b"pict"); // handler_type
    hdlr_payload.extend_from_slice(&[0; 12]); // reserved
    let hdlr = make_full_box(b"hdlr", 0, 0, &hdlr_payload);

    // pitm
    let pitm = make_full_box(b"pitm", 0, 0, &1u16.to_be_bytes());

    // infe
    let mut infe_payload = Vec::new();
    infe_payload.extend_from_slice(&1u16.to_be_bytes()); // item_id 1
    infe_payload.extend_from_slice(&0u16.to_be_bytes()); // protection 0
    infe_payload.extend_from_slice(b"hvc1"); // item_type
    infe_payload.push(0); // null-terminated item_name
    let infe = make_full_box(b"infe", 2, 0, &infe_payload);

    // iinf
    let mut iinf_payload = Vec::new();
    iinf_payload.extend_from_slice(&1u16.to_be_bytes()); // 1 entry
    iinf_payload.extend_from_slice(&infe);
    let iinf = make_full_box(b"iinf", 0, 0, &iinf_payload);

    // ispe
    let mut ispe_payload = Vec::new();
    ispe_payload.extend_from_slice(&1920u32.to_be_bytes());
    ispe_payload.extend_from_slice(&1080u32.to_be_bytes());
    let ispe = make_full_box(b"ispe", 0, 0, &ispe_payload);

    // ipco
    let ipco = make_box(b"ipco", &ispe);

    // ipma
    let mut ipma_payload = Vec::new();
    ipma_payload.extend_from_slice(&1u32.to_be_bytes()); // 1 entry
    ipma_payload.extend_from_slice(&1u16.to_be_bytes()); // item 1
    ipma_payload.push(1); // 1 association
    ipma_payload.push(1); // index 1 (ispe)
    let ipma = make_full_box(b"ipma", 0, 0, &ipma_payload);

    // iprp
    let mut iprp_payload = Vec::new();
    iprp_payload.extend_from_slice(&ipco);
    iprp_payload.extend_from_slice(&ipma);
    let iprp = make_box(b"iprp", &iprp_payload);

    // iloc placeholder
    let mut iloc_payload = Vec::new();
    iloc_payload.push(0x44); // offset_size=4, length_size=4
    iloc_payload.push(0x00); // base_offset_size=0, index_size=0
    iloc_payload.extend_from_slice(&1u16.to_be_bytes()); // 1 item
    iloc_payload.extend_from_slice(&1u16.to_be_bytes()); // item_id 1
    iloc_payload.extend_from_slice(&0u16.to_be_bytes()); // data_ref 0
    iloc_payload.extend_from_slice(&1u16.to_be_bytes()); // 1 extent
    let offset_placeholder_pos = iloc_payload.len();
    iloc_payload.extend_from_slice(&0u32.to_be_bytes()); // offset (to be computed)
    iloc_payload.extend_from_slice(&10u32.to_be_bytes()); // length 10

    // Build meta content and compute mdat offset
    let mut meta_content = Vec::new();
    meta_content.extend_from_slice(&hdlr);
    meta_content.extend_from_slice(&pitm);
    meta_content.extend_from_slice(&iinf);
    meta_content.extend_from_slice(&iprp);

    // Total meta size = 12 (meta header) + meta_content.len() + 12 (iloc header) + iloc_payload.len()
    let meta_total_size = 12 + meta_content.len() + 12 + iloc_payload.len();
    let mdat_payload_offset = ftyp.len() + meta_total_size + 8; // +8 for mdat box header

    // Update iloc with mdat offset
    iloc_payload[offset_placeholder_pos..offset_placeholder_pos + 4]
        .copy_from_slice(&(mdat_payload_offset as u32).to_be_bytes());
    let iloc = make_full_box(b"iloc", 0, 0, &iloc_payload);

    meta_content.extend_from_slice(&iloc);
    let meta = make_full_box(b"meta", 0, 0, &meta_content);

    // 3. mdat
    let mdat_payload = vec![0xAB; 10];
    let mdat = make_box(b"mdat", &mdat_payload);

    let mut file_bytes = Vec::new();
    file_bytes.extend_from_slice(&ftyp);
    file_bytes.extend_from_slice(&meta);
    file_bytes.extend_from_slice(&mdat);
    file_bytes
}

fn build_synthetic_grid_heic() -> Vec<u8> {
    let mut ftyp_payload = Vec::new();
    ftyp_payload.extend_from_slice(b"heic");
    ftyp_payload.extend_from_slice(&[0, 0, 0, 0]);
    ftyp_payload.extend_from_slice(b"mif1");
    let ftyp = make_box(b"ftyp", &ftyp_payload);

    let mut hdlr_payload = Vec::new();
    hdlr_payload.extend_from_slice(&[0; 4]);
    hdlr_payload.extend_from_slice(b"pict");
    hdlr_payload.extend_from_slice(&[0; 12]);
    let hdlr = make_full_box(b"hdlr", 0, 0, &hdlr_payload);

    let pitm = make_full_box(b"pitm", 0, 0, &1u16.to_be_bytes()); // primary is item 1 (grid)

    // Item 1 = grid, Item 2 = tile 1, Item 3 = tile 2
    let infe1 = make_full_box(b"infe", 2, 0, &[0, 1, 0, 0, b'g', b'r', b'i', b'd', 0]);
    let infe2 = make_full_box(b"infe", 2, 0, &[0, 2, 0, 0, b'h', b'v', b'c', b'1', 0]);
    let infe3 = make_full_box(b"infe", 2, 0, &[0, 3, 0, 0, b'h', b'v', b'c', b'1', 0]);

    let mut iinf_payload = Vec::new();
    iinf_payload.extend_from_slice(&3u16.to_be_bytes());
    iinf_payload.extend_from_slice(&infe1);
    iinf_payload.extend_from_slice(&infe2);
    iinf_payload.extend_from_slice(&infe3);
    let iinf = make_full_box(b"iinf", 0, 0, &iinf_payload);

    // iref: item 1 (grid) references item 2 and item 3 via 'dimg'
    let mut dimg_payload = Vec::new();
    dimg_payload.extend_from_slice(&1u16.to_be_bytes()); // from_item_id = 1
    dimg_payload.extend_from_slice(&2u16.to_be_bytes()); // 2 references
    dimg_payload.extend_from_slice(&2u16.to_be_bytes()); // to_item_id 2
    dimg_payload.extend_from_slice(&3u16.to_be_bytes()); // to_item_id 3
    let dimg_box = make_box(b"dimg", &dimg_payload);
    let iref = make_full_box(b"iref", 0, 0, &dimg_box);

    // ispe for item 1 (grid: 1000x500)
    let mut ispe_payload = Vec::new();
    ispe_payload.extend_from_slice(&1000u32.to_be_bytes());
    ispe_payload.extend_from_slice(&500u32.to_be_bytes());
    let ispe = make_full_box(b"ispe", 0, 0, &ispe_payload);

    let ipco = make_box(b"ipco", &ispe);

    let mut ipma_payload = Vec::new();
    ipma_payload.extend_from_slice(&1u32.to_be_bytes()); // 1 entry
    ipma_payload.extend_from_slice(&1u16.to_be_bytes()); // item 1
    ipma_payload.push(1); // 1 assoc
    ipma_payload.push(1); // index 1
    let ipma = make_full_box(b"ipma", 0, 0, &ipma_payload);

    let mut iprp_payload = Vec::new();
    iprp_payload.extend_from_slice(&ipco);
    iprp_payload.extend_from_slice(&ipma);
    let iprp = make_box(b"iprp", &iprp_payload);

    // iloc with 3 items
    let mut iloc_payload = Vec::new();
    iloc_payload.push(0x44);
    iloc_payload.push(0x00);
    iloc_payload.extend_from_slice(&3u16.to_be_bytes()); // 3 items

    // item 1 (grid descriptor: 8 bytes)
    iloc_payload.extend_from_slice(&1u16.to_be_bytes());
    iloc_payload.extend_from_slice(&0u16.to_be_bytes());
    iloc_payload.extend_from_slice(&1u16.to_be_bytes());
    let offset_grid_pos = iloc_payload.len();
    iloc_payload.extend_from_slice(&0u32.to_be_bytes());
    iloc_payload.extend_from_slice(&8u32.to_be_bytes());

    // item 2 (tile 1: 10 bytes)
    iloc_payload.extend_from_slice(&2u16.to_be_bytes());
    iloc_payload.extend_from_slice(&0u16.to_be_bytes());
    iloc_payload.extend_from_slice(&1u16.to_be_bytes());
    let offset_t1_pos = iloc_payload.len();
    iloc_payload.extend_from_slice(&0u32.to_be_bytes());
    iloc_payload.extend_from_slice(&10u32.to_be_bytes());

    // item 3 (tile 2: 10 bytes)
    iloc_payload.extend_from_slice(&3u16.to_be_bytes());
    iloc_payload.extend_from_slice(&0u16.to_be_bytes());
    iloc_payload.extend_from_slice(&1u16.to_be_bytes());
    let offset_t2_pos = iloc_payload.len();
    iloc_payload.extend_from_slice(&0u32.to_be_bytes());
    iloc_payload.extend_from_slice(&10u32.to_be_bytes());

    let mut meta_content = Vec::new();
    meta_content.extend_from_slice(&hdlr);
    meta_content.extend_from_slice(&pitm);
    meta_content.extend_from_slice(&iinf);
    meta_content.extend_from_slice(&iref);
    meta_content.extend_from_slice(&iprp);

    let meta_total_size = 12 + meta_content.len() + 12 + iloc_payload.len();
    let mdat_payload_offset = ftyp.len() + meta_total_size + 8;

    iloc_payload[offset_grid_pos..offset_grid_pos + 4]
        .copy_from_slice(&(mdat_payload_offset as u32).to_be_bytes());
    iloc_payload[offset_t1_pos..offset_t1_pos + 4]
        .copy_from_slice(&((mdat_payload_offset + 8) as u32).to_be_bytes());
    iloc_payload[offset_t2_pos..offset_t2_pos + 4]
        .copy_from_slice(&((mdat_payload_offset + 18) as u32).to_be_bytes());

    let iloc = make_full_box(b"iloc", 0, 0, &iloc_payload);
    meta_content.extend_from_slice(&iloc);
    let meta = make_full_box(b"meta", 0, 0, &meta_content);

    // mdat: 8 bytes grid (0, 0, rows_minus1=0, cols_minus1=1, width=1000, height=500) + 10 bytes tile1 + 10 bytes tile2
    let mut mdat_payload = Vec::new();
    // grid payload (version=0, flags=0, rows_minus_one=0 (1 row), cols_minus_one=1 (2 cols), w=1000, h=500)
    mdat_payload.extend_from_slice(&[0, 0, 0, 1]); // version 0, flags 0, rows-1=0, cols-1=1
    mdat_payload.extend_from_slice(&1000u16.to_be_bytes());
    mdat_payload.extend_from_slice(&500u16.to_be_bytes());
    mdat_payload.extend_from_slice(&[0x11; 10]); // tile 1
    mdat_payload.extend_from_slice(&[0x22; 10]); // tile 2
    let mdat = make_box(b"mdat", &mdat_payload);

    let mut file_bytes = Vec::new();
    file_bytes.extend_from_slice(&ftyp);
    file_bytes.extend_from_slice(&meta);
    file_bytes.extend_from_slice(&mdat);
    file_bytes
}

#[test]
fn test_is_heif_or_heic_valid_ftyp() {
    let mut data = vec![0, 0, 0, 16, b'f', b't', b'y', b'p'];
    data.extend_from_slice(b"heic");
    data.extend_from_slice(&[0, 0, 0, 0]);

    assert!(is_heif_or_heic(&data));
}

#[test]
fn test_is_heif_or_heic_invalid() {
    assert!(!is_heif_or_heic(&[]));
    assert!(!is_heif_or_heic(b"random_data_here"));
}

#[test]
fn test_parse_synthetic_heic_pipeline() {
    let data = build_synthetic_heic();
    let limits = Limits::default();

    let heif = parse_heif(&data, &limits).expect("Synthetic HEIC parse should succeed");
    assert_eq!(heif.ftyp.major_brand, *b"heic");
    assert_eq!(heif.primary_item_id, 1);

    let primary = heif
        .items
        .get(&heif.primary_item_id)
        .expect("Primary image should exist");
    assert_eq!(primary.dimensions.width, 1920);
    assert_eq!(primary.dimensions.height, 1080);
    assert_eq!(primary.item_type, valen_heif_parser::boxes::FourCC::HVC1);

    let primary_data = heif.extract_item_data(&data, heif.primary_item_id).unwrap();
    assert_eq!(primary_data, vec![0xAB; 10]);

    let meta = inspect_container(&data, &limits).expect("Inspect should succeed");
    assert_eq!(meta.dimensions.width, 1920);
    assert_eq!(meta.dimensions.height, 1080);
    assert!(!meta.is_grid);
}

#[test]
fn test_parse_synthetic_grid_heic() {
    let data = build_synthetic_grid_heic();
    let limits = Limits::default();

    let heif = parse_heif(&data, &limits).expect("Synthetic Grid HEIC parse should succeed");
    assert!(heif.grid_config.is_some());
    assert_eq!(&heif.grid_tile_item_ids, &[2, 3]);

    let meta = inspect_container(&data, &limits).expect("Inspect should succeed");
    assert!(meta.is_grid);
    assert_eq!(meta.grid_rows, 1);
    assert_eq!(meta.grid_columns, 2);
    assert_eq!(meta.dimensions.width, 1000);
    assert_eq!(meta.dimensions.height, 500);
}

#[test]
fn test_nal_stream_annex_b_extraction() {
    let config = HevcConfigProperty {
        nalu_length_size: 4,
        sps: vec![vec![0x42, 0x01, 0x01]],
        pps: vec![vec![0x44, 0x01, 0xC0]],
        vps: vec![vec![0x40, 0x01, 0x0C]],
    };

    let annex_b_header = config.to_annex_b_header();
    // Should start with Annex-B start code [0, 0, 0, 1]
    assert_eq!(&annex_b_header[0..4], &[0x00, 0x00, 0x00, 0x01]);
}
