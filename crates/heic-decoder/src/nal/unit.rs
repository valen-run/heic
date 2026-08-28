//! NAL unit types and Annex-B stream splitting.

use valen_heic_core::{HeicError, HeicResult};

/// HEVC Network Abstraction Layer Unit (NALU) types (ITU-T H.265 Table 7-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NalUnitType {
    /// Trailing sub-layer non-reference picture.
    TrailN,
    /// Trailing sub-layer reference picture.
    TrailR,
    /// Broken Link Access with RADL pictures.
    BlaWRadl,
    /// Broken Link Access with Decodable Leading pictures.
    BlaWDec,
    /// Broken Link Access non-leading pictures.
    BlaNLp,
    /// Instantaneous Decoding Refresh with RADL pictures.
    IdrWRadl,
    /// Instantaneous Decoding Refresh non-leading pictures.
    IdrNLp,
    /// Clean Random Access picture.
    CraNut,
    /// Video Parameter Set.
    VpsNut,
    /// Sequence Parameter Set.
    SpsNut,
    /// Picture Parameter Set.
    PpsNut,
    /// Access Unit Delimiter.
    AudNut,
    /// End of Sequence.
    EosNut,
    /// End of Bitstream.
    EobNut,
    /// Filler Data.
    FdNut,
    /// Supplemental Enhancement Information (Prefix).
    PrefixSeiNut,
    /// Supplemental Enhancement Information (Suffix).
    SuffixSeiNut,
    /// Other or unreserved NAL unit type.
    Other(u8),
}

impl NalUnitType {
    /// Creates a [`NalUnitType`] from a 6-bit numeric identifier.
    pub const fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::TrailN,
            1 => Self::TrailR,
            16 => Self::BlaWRadl,
            17 => Self::BlaWDec,
            18 => Self::BlaNLp,
            19 => Self::IdrWRadl,
            20 => Self::IdrNLp,
            21 => Self::CraNut,
            32 => Self::VpsNut,
            33 => Self::SpsNut,
            34 => Self::PpsNut,
            35 => Self::AudNut,
            36 => Self::EosNut,
            37 => Self::EobNut,
            38 => Self::FdNut,
            39 => Self::PrefixSeiNut,
            40 => Self::SuffixSeiNut,
            other => Self::Other(other),
        }
    }

    /// Returns `true` if this NAL unit represents an Intra/IRAP (Intra Random Access Point) slice.
    pub const fn is_irap(&self) -> bool {
        matches!(
            self,
            Self::BlaWRadl
                | Self::BlaWDec
                | Self::BlaNLp
                | Self::IdrWRadl
                | Self::IdrNLp
                | Self::CraNut
        )
    }

    /// Returns `true` if this NAL unit contains slice segment data.
    pub const fn is_slice(&self) -> bool {
        matches!(
            self,
            Self::TrailN
                | Self::TrailR
                | Self::BlaWRadl
                | Self::BlaWDec
                | Self::BlaNLp
                | Self::IdrWRadl
                | Self::IdrNLp
                | Self::CraNut
        )
    }
}

/// Single parsed NAL unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NalUnit<'a> {
    /// NAL unit type.
    pub unit_type: NalUnitType,
    /// Layer identifier (typically 0).
    pub layer_id: u8,
    /// Temporal sub-layer identifier minus 1.
    pub temporal_id_plus1: u8,
    /// Unescaped Raw Byte Sequence Payload (RBSP) data.
    pub rbsp_data: Vec<u8>,
    /// Raw slice reference into input bitstream.
    pub raw_bytes: &'a [u8],
}

impl<'a> NalUnit<'a> {
    /// Splits an Annex-B bitstream into individual NAL units.
    pub fn parse_annex_b(input: &'a [u8]) -> HeicResult<Vec<NalUnit<'a>>> {
        let mut units = Vec::new();
        let mut cursor = 0;

        while cursor < input.len() {
            // Find start code (0x000001 or 0x00000001)
            let mut start_code_len = 0;
            if cursor + 3 <= input.len()
                && input[cursor] == 0
                && input[cursor + 1] == 0
                && input[cursor + 2] == 1
            {
                start_code_len = 3;
            } else if cursor + 4 <= input.len()
                && input[cursor] == 0
                && input[cursor + 1] == 0
                && input[cursor + 2] == 0
                && input[cursor + 3] == 1
            {
                start_code_len = 4;
            }

            if start_code_len == 0 {
                cursor += 1;
                continue;
            }

            let start = cursor + start_code_len;
            // Find next start code or end of buffer
            let mut end = input.len();
            let mut search = start;
            while search + 3 <= input.len() {
                let is_4byte_start = search + 4 <= input.len()
                    && input[search] == 0
                    && input[search + 1] == 0
                    && input[search + 2] == 0
                    && input[search + 3] == 1;
                let is_3byte_start =
                    input[search] == 0 && input[search + 1] == 0 && input[search + 2] == 1;

                if is_4byte_start || is_3byte_start {
                    end = search;
                    break;
                }
                search += 1;
            }

            if start < end {
                let nalu_slice = &input[start..end];
                if nalu_slice.len() >= 2 {
                    let header0 = nalu_slice[0];
                    let header1 = nalu_slice[1];

                    let forbidden_zero_bit = (header0 >> 7) & 1;
                    if forbidden_zero_bit != 0 {
                        return Err(HeicError::MalformedInput(
                            "Invalid NAL unit: forbidden_zero_bit is 1".into(),
                        ));
                    }

                    let nal_type_val = (header0 >> 1) & 0x3F;
                    let layer_id = ((header0 & 1) << 5) | ((header1 >> 3) & 0x1F);
                    let temporal_id_plus1 = header1 & 0x07;

                    let payload = &nalu_slice[2..];
                    let rbsp_data = remove_emulation_prevention_bytes(payload);

                    units.push(NalUnit {
                        unit_type: NalUnitType::from_u8(nal_type_val),
                        layer_id,
                        temporal_id_plus1,
                        rbsp_data,
                        raw_bytes: nalu_slice,
                    });
                }
            }

            cursor = end;
        }

        Ok(units)
    }
}

/// Removes emulation prevention bytes (`0x00 0x00 0x03 -> 0x00 0x00`) from NAL payload to form RBSP.
pub fn remove_emulation_prevention_bytes(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if i + 2 < input.len() && input[i] == 0 && input[i + 1] == 0 && input[i + 2] == 3 {
            output.push(0);
            output.push(0);
            i += 3;
        } else {
            output.push(input[i]);
            i += 1;
        }
    }
    output
}
