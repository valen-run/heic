//! RIFF container framing for WebP files.

/// Wraps VP8L payload into a standard RIFF/WEBP container format.
pub fn wrap_riff_webp(vp8l_data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(12 + 8 + vp8l_data.len() + 2);

    let vp8l_chunk_len = vp8l_data.len() as u32;
    let riff_len = 4 + 8 + vp8l_chunk_len + (vp8l_chunk_len & 1);

    // 'RIFF' + size + 'WEBP'
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_len.to_le_bytes());
    out.extend_from_slice(b"WEBP");

    // 'VP8L' + size + data
    out.extend_from_slice(b"VP8L");
    out.extend_from_slice(&vp8l_chunk_len.to_le_bytes());
    out.extend_from_slice(vp8l_data);

    if (vp8l_chunk_len & 1) != 0 {
        out.push(0x00); // RIFF 2-byte alignment padding
    }

    out
}
