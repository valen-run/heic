//! Pure-Rust Zlib Deflate container encoding (RFC 1950 & 1951).

/// Computes the IEEE 802.3 32-bit Cyclic Redundancy Check (CRC-32).
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// Computes the Adler-32 checksum (RFC 1950).
pub fn adler32(data: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65521;
    let mut s1 = 1u32;
    let mut s2 = 0u32;

    for &byte in data {
        s1 = (s1 + byte as u32) % MOD_ADLER;
        s2 = (s2 + s1) % MOD_ADLER;
    }

    (s2 << 16) | s1
}

/// Encodes uncompressed filtered scanlines into a Zlib Deflate container (RFC 1950 & 1951).
pub fn deflate_zlib(uncompressed: &[u8]) -> Vec<u8> {
    let mut zlib = Vec::with_capacity(uncompressed.len() + 64);

    // 1. Zlib Header (CMF = 0x78 (Deflate, 32KB window), FLG = 0x01 (No preset dict, check bits))
    zlib.push(0x78);
    zlib.push(0x01);

    // 2. Deflate Non-compressed Blocks (BTYPE = 00)
    let chunk_size = 65535;
    let chunks: Vec<&[u8]> = uncompressed.chunks(chunk_size).collect();

    if chunks.is_empty() {
        zlib.push(0x01);
        zlib.extend_from_slice(&[0x00, 0x00, 0xFF, 0xFF]);
    } else {
        for (i, chunk) in chunks.iter().enumerate() {
            let is_final = i == chunks.len() - 1;
            let bfinal_btype = if is_final { 0x01 } else { 0x00 };
            zlib.push(bfinal_btype);

            let len = chunk.len() as u16;
            let nlen = !len;

            zlib.push(len as u8);
            zlib.push((len >> 8) as u8);
            zlib.push(nlen as u8);
            zlib.push((nlen >> 8) as u8);

            zlib.extend_from_slice(chunk);
        }
    }

    // 3. Adler-32 Checksum (big-endian)
    let adler = adler32(uncompressed);
    zlib.extend_from_slice(&adler.to_be_bytes());

    zlib
}
