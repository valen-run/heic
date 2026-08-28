//! HEVC decoder configuration property record (`hvcC`).

/// HEVC configuration record parsed from `hvcC`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HevcConfigProperty {
    /// NAL unit length size in bytes (typically 4).
    pub nalu_length_size: u8,
    /// Sequence Parameter Sets (SPS).
    pub sps: Vec<Vec<u8>>,
    /// Picture Parameter Sets (PPS).
    pub pps: Vec<Vec<u8>>,
    /// Video Parameter Sets (VPS).
    pub vps: Vec<Vec<u8>>,
}

impl HevcConfigProperty {
    /// Formats all VPS, SPS, and PPS parameter sets into Annex-B start-code prefixed bitstream bytes.
    pub fn to_annex_b_header(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for vps in &self.vps {
            out.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
            out.extend_from_slice(vps);
        }
        for sps in &self.sps {
            out.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
            out.extend_from_slice(sps);
        }
        for pps in &self.pps {
            out.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
            out.extend_from_slice(pps);
        }
        out
    }
}
