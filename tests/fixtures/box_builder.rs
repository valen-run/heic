//! ISO-BMFF low-level box and full box serialization helpers for test fixtures.

/// Constructs an ISOBMFF box with 4-byte size header and 4-byte FourCC type.
pub fn make_box(fourcc: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let size = (8 + payload.len()) as u32;
    let mut b = Vec::with_capacity(size as usize);
    b.extend_from_slice(&size.to_be_bytes());
    b.extend_from_slice(fourcc);
    b.extend_from_slice(payload);
    b
}

/// Constructs an ISOBMFF FullBox with version and 24-bit flags.
pub fn make_full_box(fourcc: &[u8; 4], version: u8, flags: u32, payload: &[u8]) -> Vec<u8> {
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

/// Constructs a standard `ftyp` box.
pub fn make_ftyp(
    major_brand: &[u8; 4],
    minor_version: u32,
    compatible_brands: &[&[u8; 4]],
) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(major_brand);
    payload.extend_from_slice(&minor_version.to_be_bytes());
    for &brand in compatible_brands {
        payload.extend_from_slice(brand);
    }
    make_box(b"ftyp", &payload)
}

/// Constructs an `hdlr` handler box with `pict` handler type.
pub fn make_hdlr() -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&[0; 4]); // pre_defined
    payload.extend_from_slice(b"pict"); // handler_type
    payload.extend_from_slice(&[0; 12]); // reserved
    make_full_box(b"hdlr", 0, 0, &payload)
}

/// Constructs a `pitm` primary item box.
pub fn make_pitm(primary_item_id: u16) -> Vec<u8> {
    make_full_box(b"pitm", 0, 0, &primary_item_id.to_be_bytes())
}

/// Constructs an `infe` item info entry box (version 2).
pub fn make_infe(item_id: u16, item_type: &[u8; 4]) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&item_id.to_be_bytes());
    payload.extend_from_slice(&0u16.to_be_bytes()); // protection_index
    payload.extend_from_slice(item_type);
    payload.push(0); // null-terminated item_name
    make_full_box(b"infe", 2, 0, &payload)
}

/// Constructs an `iinf` item info box from a slice of `infe` box bytes.
pub fn make_iinf(infe_boxes: &[Vec<u8>]) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&(infe_boxes.len() as u16).to_be_bytes());
    for infe in infe_boxes {
        payload.extend_from_slice(infe);
    }
    make_full_box(b"iinf", 0, 0, &payload)
}

/// Constructs an `ispe` spatial extents property box.
pub fn make_ispe(width: u32, height: u32) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&width.to_be_bytes());
    payload.extend_from_slice(&height.to_be_bytes());
    make_full_box(b"ispe", 0, 0, &payload)
}

/// Constructs an `irot` rotation property box (0=0°, 1=90° CCW, 2=180°, 3=270° CCW).
pub fn make_irot(angle_ccw: u8) -> Vec<u8> {
    make_box(b"irot", &[angle_ccw & 3])
}

/// Constructs an `imir` mirror property box (0=vertical axis flip, 1=horizontal axis flip).
pub fn make_imir(axis: u8) -> Vec<u8> {
    make_box(b"imir", &[axis & 1])
}

/// Constructs a `colr` NCLX color property box.
pub fn make_colr_nclx(primaries: u16, transfer: u16, matrix: u16, full_range: bool) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(b"nclx");
    payload.extend_from_slice(&primaries.to_be_bytes());
    payload.extend_from_slice(&transfer.to_be_bytes());
    payload.extend_from_slice(&matrix.to_be_bytes());
    payload.push(if full_range { 0x80 } else { 0x00 });
    make_box(b"colr", &payload)
}

/// Constructs an `auxC` auxiliary property box.
pub fn make_auxc(aux_type: &str) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(aux_type.as_bytes());
    payload.push(0); // null-terminated
    make_full_box(b"auxC", 0, 0, &payload)
}

/// Constructs an `ipma` item property association box.
pub fn make_ipma(associations: &[(u16, &[u8])]) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&(associations.len() as u32).to_be_bytes());
    for &(item_id, prop_indices) in associations {
        payload.extend_from_slice(&item_id.to_be_bytes());
        payload.push(prop_indices.len() as u8);
        for &idx in prop_indices {
            payload.push(idx);
        }
    }
    make_full_box(b"ipma", 0, 0, &payload)
}

/// Constructs an `iprp` item properties box.
pub fn make_iprp(properties: &[Vec<u8>], associations: &[(u16, &[u8])]) -> Vec<u8> {
    let mut ipco_payload = Vec::new();
    for prop in properties {
        ipco_payload.extend_from_slice(prop);
    }
    let ipco = make_box(b"ipco", &ipco_payload);
    let ipma = make_ipma(associations);

    let mut iprp_payload = Vec::new();
    iprp_payload.extend_from_slice(&ipco);
    iprp_payload.extend_from_slice(&ipma);
    make_box(b"iprp", &iprp_payload)
}

/// Constructs an `iloc` item location box (offset_size=4, length_size=4).
pub fn make_iloc(items: &[(u16, u32, u32)]) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.push(0x44); // offset_size=4, length_size=4
    payload.push(0x00); // base_offset_size=0, index_size=0
    payload.extend_from_slice(&(items.len() as u16).to_be_bytes());

    for &(item_id, offset, length) in items {
        payload.extend_from_slice(&item_id.to_be_bytes());
        payload.extend_from_slice(&0u16.to_be_bytes()); // data_reference_index
        payload.extend_from_slice(&1u16.to_be_bytes()); // extent_count = 1
        payload.extend_from_slice(&offset.to_be_bytes());
        payload.extend_from_slice(&length.to_be_bytes());
    }
    make_full_box(b"iloc", 0, 0, &payload)
}

/// Constructs an `iref` item reference box with a reference type and targets.
pub fn make_iref(ref_type: &[u8; 4], from_item_id: u16, to_item_ids: &[u16]) -> Vec<u8> {
    let mut typed_payload = Vec::new();
    typed_payload.extend_from_slice(&from_item_id.to_be_bytes());
    typed_payload.extend_from_slice(&(to_item_ids.len() as u16).to_be_bytes());
    for &to_id in to_item_ids {
        typed_payload.extend_from_slice(&to_id.to_be_bytes());
    }
    let typed_box = make_box(ref_type, &typed_payload);
    make_full_box(b"iref", 0, 0, &typed_box)
}
