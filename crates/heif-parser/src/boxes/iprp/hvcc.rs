//! `hvcC` HEVC Decoder Configuration Box parser.

use crate::boxes::iprp::properties::HevcConfigProperty;
use valen_heic_core::{HeicError, HeicResult};

/// Parses `hvcC` payload into `HevcConfigProperty`.
pub fn parse_hvcc(payload: &[u8]) -> HeicResult<HevcConfigProperty> {
    if payload.len() < 22 {
        return Err(HeicError::MalformedInput("Truncated hvcC payload".into()));
    }

    let length_size_minus_one = payload[21] & 3;
    let nalu_length_size = length_size_minus_one + 1;
    let num_of_arrays = payload[22] as usize;

    let mut sps = Vec::new();
    let mut pps = Vec::new();
    let mut vps = Vec::new();

    let mut cursor = 23;
    for _ in 0..num_of_arrays {
        if payload.len() < cursor + 3 {
            break;
        }
        let nal_unit_type = payload[cursor] & 0x3F;
        let num_nalus = u16::from_be_bytes([payload[cursor + 1], payload[cursor + 2]]) as usize;
        cursor += 3;

        for _ in 0..num_nalus {
            if payload.len() < cursor + 2 {
                break;
            }
            let nalu_len = u16::from_be_bytes([payload[cursor], payload[cursor + 1]]) as usize;
            cursor += 2;

            if payload.len() < cursor + nalu_len {
                break;
            }
            let nalu_bytes = payload[cursor..cursor + nalu_len].to_vec();
            cursor += nalu_len;

            match nal_unit_type {
                32 => vps.push(nalu_bytes),
                33 => sps.push(nalu_bytes),
                34 => pps.push(nalu_bytes),
                _ => {}
            }
        }
    }

    Ok(HevcConfigProperty {
        nalu_length_size,
        sps,
        pps,
        vps,
    })
}
