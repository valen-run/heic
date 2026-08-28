//! PNG Chunk framing and serialization.

use super::deflate::crc32;

/// Writes a PNG chunk with length, chunk type, data, and CRC-32.
pub fn write_chunk(dest: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    let len = data.len() as u32;
    dest.extend_from_slice(&len.to_be_bytes());
    dest.extend_from_slice(chunk_type);
    dest.extend_from_slice(data);

    let mut crc_payload = Vec::with_capacity(4 + data.len());
    crc_payload.extend_from_slice(chunk_type);
    crc_payload.extend_from_slice(data);

    let crc = crc32(&crc_payload);
    dest.extend_from_slice(&crc.to_be_bytes());
}

/// Writes the 13-byte IHDR image header chunk.
pub fn write_ihdr(dest: &mut Vec<u8>, width: u32, height: u32, color_type: u8) {
    let mut ihdr_data = [0u8; 13];
    ihdr_data[0..4].copy_from_slice(&width.to_be_bytes());
    ihdr_data[4..8].copy_from_slice(&height.to_be_bytes());
    ihdr_data[8] = 8; // Bit depth: 8 bits per channel
    ihdr_data[9] = color_type;
    ihdr_data[10] = 0; // Compression method: 0 (Deflate)
    ihdr_data[11] = 0; // Filter method: 0 (Adaptive)
    ihdr_data[12] = 0; // Interlace method: 0 (None)

    write_chunk(dest, b"IHDR", &ihdr_data);
}
