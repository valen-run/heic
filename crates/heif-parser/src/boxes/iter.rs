//! Iterator over sibling ISOBMFF boxes.

use super::header::BoxHeader;
use valen_heic_core::{HeicError, HeicResult};

/// Iterator over sibling boxes in a container payload.
pub struct BoxIter<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> BoxIter<'a> {
    /// Creates a new box iterator over a byte slice.
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }
}

impl<'a> Iterator for BoxIter<'a> {
    type Item = HeicResult<(BoxHeader, &'a [u8])>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.data.len() {
            return None;
        }

        let remaining = &self.data[self.offset..];
        if remaining.len() < 8 {
            self.offset = self.data.len();
            return Some(Err(HeicError::MalformedInput(
                "Trailing bytes insufficient for box header".to_string(),
            )));
        }

        let header = match BoxHeader::parse(remaining) {
            Ok(h) => h,
            Err(e) => {
                self.offset = self.data.len();
                return Some(Err(e));
            }
        };

        let box_size = match usize::try_from(header.size) {
            Ok(s) => s,
            Err(_) => {
                self.offset = self.data.len();
                return Some(Err(HeicError::LimitExceeded(
                    "Box size exceeds usize".to_string(),
                )));
            }
        };

        if remaining.len() < box_size {
            self.offset = self.data.len();
            return Some(Err(HeicError::MalformedInput(format!(
                "Box {} declares {} bytes but only {} remain",
                header.box_type,
                box_size,
                remaining.len()
            ))));
        }

        let box_data = &remaining[..box_size];
        self.offset += box_size;
        Some(Ok((header, box_data)))
    }
}
