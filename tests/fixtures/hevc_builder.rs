//! Minimal synthetic HEVC NAL bitstream generation for test fixtures.

/// Builds an Annex-B formatted NAL stream with `0x00000001` 4-byte start codes.
pub fn build_annex_b_stream(nalus: &[&[u8]]) -> Vec<u8> {
    let mut stream = Vec::new();
    for nalu in nalus {
        stream.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
        stream.extend_from_slice(nalu);
    }
    stream
}

/// Builds an MP4 length-prefixed NAL stream (4-byte length headers).
pub fn build_length_prefixed_stream(nalus: &[&[u8]]) -> Vec<u8> {
    let mut stream = Vec::new();
    for nalu in nalus {
        let len = nalu.len() as u32;
        stream.extend_from_slice(&len.to_be_bytes());
        stream.extend_from_slice(nalu);
    }
    stream
}

/// Generates a mock VPS NAL unit.
pub fn mock_vps_nal() -> &'static [u8] {
    &[0x40, 0x01, 0x0C, 0x01, 0xFF, 0xFF, 0x01, 0x60, 0x00, 0x00]
}

/// Generates a mock SPS NAL unit.
pub fn mock_sps_nal() -> &'static [u8] {
    &[0x42, 0x01, 0x01, 0x01, 0x60, 0x00, 0x00, 0x03, 0x00, 0xB0]
}

/// Generates a mock PPS NAL unit.
pub fn mock_pps_nal() -> &'static [u8] {
    &[0x44, 0x01, 0xC0, 0xF2, 0xC0]
}

/// Generates a mock Intra Slice NAL unit payload.
pub fn mock_slice_nal(payload_len: usize) -> Vec<u8> {
    let mut slice = vec![0x26, 0x01, 0xAF]; // IDR_N_LP header
    slice.resize(payload_len.max(3), 0x55);
    slice
}

/// Generates a complete mock HEVC Annex-B bitstream with VPS, SPS, PPS, and Slice.
pub fn mock_hevc_annex_b(slice_size: usize) -> Vec<u8> {
    let slice = mock_slice_nal(slice_size);
    build_annex_b_stream(&[mock_vps_nal(), mock_sps_nal(), mock_pps_nal(), &slice])
}
