//! ISOBMFF Box header structures (standard 8-byte, extended 16-byte, and FullBox).

use super::FourCC;
use valen_heic_core::{HeicError, HeicResult};

/// Generic ISOBMFF Box header (8-byte standard or 16-byte extended).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoxHeader {
    /// The four-character code of this box.
    pub box_type: FourCC,
    /// Total box size in bytes including header.
    pub size: u64,
    /// Header size in bytes (8 for standard 32-bit, 16 for extended 64-bit).
    pub header_size: usize,
}

impl BoxHeader {
    /// Parses a box header from the beginning of a byte slice.
    pub fn parse(input: &[u8]) -> HeicResult<Self> {
        if input.len() < 8 {
            return Err(HeicError::MalformedInput(
                "Insufficient bytes for ISOBMFF box header (need >= 8 bytes)".to_string(),
            ));
        }

        let size_raw = u32::from_be_bytes([input[0], input[1], input[2], input[3]]);
        let box_type = FourCC([input[4], input[5], input[6], input[7]]);

        if size_raw == 1 {
            // 64-bit extended size
            if input.len() < 16 {
                return Err(HeicError::MalformedInput(
                    "Insufficient bytes for extended 64-bit box header (need >= 16 bytes)"
                        .to_string(),
                ));
            }
            let extended_size = u64::from_be_bytes([
                input[8], input[9], input[10], input[11], input[12], input[13], input[14],
                input[15],
            ]);
            if extended_size < 16 {
                return Err(HeicError::InvalidContainer(format!(
                    "Invalid 64-bit box size {extended_size} < header size 16 for {box_type}"
                )));
            }
            Ok(Self {
                box_type,
                size: extended_size,
                header_size: 16,
            })
        } else if size_raw == 0 {
            // Box extends to end of input
            Ok(Self {
                box_type,
                size: input.len() as u64,
                header_size: 8,
            })
        } else {
            if (size_raw as usize) < 8 {
                return Err(HeicError::InvalidContainer(format!(
                    "Invalid box size {size_raw} < header size 8 for {box_type}"
                )));
            }
            Ok(Self {
                box_type,
                size: size_raw as u64,
                header_size: 8,
            })
        }
    }

    /// Computes payload size in bytes excluding header bytes.
    #[inline]
    pub const fn payload_size(&self) -> u64 {
        self.size.saturating_sub(self.header_size as u64)
    }

    /// Extracts the payload slice corresponding to this box from `input`.
    pub fn payload<'a>(&self, input: &'a [u8]) -> HeicResult<&'a [u8]> {
        let total_size = usize::try_from(self.size).map_err(|_| {
            HeicError::LimitExceeded("Box size exceeds addressable memory".to_string())
        })?;
        if input.len() < total_size {
            return Err(HeicError::MalformedInput(format!(
                "Box {} truncated: expected {} bytes, got {}",
                self.box_type,
                total_size,
                input.len()
            )));
        }
        Ok(&input[self.header_size..total_size])
    }
}

/// Full Box Header containing version and flags (ISO/IEC 14496-12 Section 4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FullBoxHeader {
    /// Base box header.
    pub header: BoxHeader,
    /// Box specification version (typically 0 or 1).
    pub version: u8,
    /// 24-bit integer flags.
    pub flags: u32,
    /// Total header length including 4 bytes of version/flags.
    pub total_header_size: usize,
}

impl FullBoxHeader {
    /// Parses a Full Box Header from input.
    pub fn parse(input: &[u8]) -> HeicResult<Self> {
        let header = BoxHeader::parse(input)?;
        let start = header.header_size;
        if input.len() < start + 4 {
            return Err(HeicError::MalformedInput(format!(
                "Insufficient bytes for FullBox header in {}",
                header.box_type
            )));
        }

        let version = input[start];
        let flags = ((input[start + 1] as u32) << 16)
            | ((input[start + 2] as u32) << 8)
            | (input[start + 3] as u32);

        Ok(Self {
            header,
            version,
            flags,
            total_header_size: start + 4,
        })
    }

    /// Extracts the payload slice excluding full box version/flags.
    pub fn payload<'a>(&self, input: &'a [u8]) -> HeicResult<&'a [u8]> {
        let total_size = usize::try_from(self.header.size).map_err(|_| {
            HeicError::LimitExceeded("Box size exceeds addressable memory".to_string())
        })?;
        if input.len() < total_size {
            return Err(HeicError::MalformedInput(format!(
                "Full box {} truncated: expected {} bytes, got {}",
                self.header.box_type,
                total_size,
                input.len()
            )));
        }
        Ok(&input[self.total_header_size..total_size])
    }
}
