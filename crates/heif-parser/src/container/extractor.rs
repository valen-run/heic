//! Bitstream and metadata extraction primitives from demuxed container boxes.

use super::HeifFile;
use crate::boxes::meta::iloc::ConstructionMethod;
use valen_heic_core::{HeicError, HeicResult};

impl HeifFile {
    /// Extracts raw payload data for an item by reading all its extents according to `iloc`.
    pub fn extract_item_data(&self, file_bytes: &[u8], item_id: u32) -> HeicResult<Vec<u8>> {
        let loc = self.meta.iloc.items.get(&item_id).ok_or_else(|| {
            HeicError::InvalidContainer(format!("Item ID {item_id} has no iloc entry"))
        })?;

        let mut output = Vec::with_capacity(usize::try_from(loc.total_length()).unwrap_or(0));

        match loc.construction_method {
            ConstructionMethod::FileOffset => {
                for extent in &loc.extents {
                    let start = usize::try_from(loc.base_offset.saturating_add(extent.offset))
                        .map_err(|_| {
                            HeicError::LimitExceeded("Extent offset exceeds usize".into())
                        })?;
                    let len = usize::try_from(extent.length).map_err(|_| {
                        HeicError::LimitExceeded("Extent length exceeds usize".into())
                    })?;
                    let end = start.saturating_add(len);

                    if end > file_bytes.len() {
                        return Err(HeicError::MalformedInput(format!(
                            "Extent [{}..{}] exceeds file length {}",
                            start,
                            end,
                            file_bytes.len()
                        )));
                    }
                    output.extend_from_slice(&file_bytes[start..end]);
                }
            }
            ConstructionMethod::IdatOffset => {
                let idat_data = self.meta.idat.as_ref().ok_or_else(|| {
                    HeicError::InvalidContainer(
                        "Item uses idat construction but meta has no idat box".into(),
                    )
                })?;

                for extent in &loc.extents {
                    let start = usize::try_from(loc.base_offset.saturating_add(extent.offset))
                        .map_err(|_| {
                            HeicError::LimitExceeded("Extent offset exceeds usize".into())
                        })?;
                    let len = usize::try_from(extent.length).map_err(|_| {
                        HeicError::LimitExceeded("Extent length exceeds usize".into())
                    })?;
                    let end = start.saturating_add(len);

                    if end > idat_data.len() {
                        return Err(HeicError::MalformedInput(format!(
                            "Extent [{}..{}] exceeds idat length {}",
                            start,
                            end,
                            idat_data.len()
                        )));
                    }
                    output.extend_from_slice(&idat_data[start..end]);
                }
            }
            ConstructionMethod::ItemOffset => {
                return Err(HeicError::UnsupportedFeature(
                    "ConstructionMethod 2 (item offset) is not yet supported".to_string(),
                ));
            }
        }

        Ok(output)
    }

    /// Extracts an Annex-B formatted HEVC bitstream (SPS/PPS/VPS headers + length-prefixed slice NAL units).
    pub fn extract_annex_b_stream(&self, file_bytes: &[u8], item_id: u32) -> HeicResult<Vec<u8>> {
        let item_data = self.extract_item_data(file_bytes, item_id)?;
        let hevc_config = self.iprp.get_hevc_config_for_item(item_id);

        let mut annex_b = Vec::new();

        let nalu_length_size = if let Some(config) = hevc_config {
            annex_b.extend_from_slice(&config.to_annex_b_header());
            config.nalu_length_size as usize
        } else {
            4
        };

        let mut offset = 0;
        while offset + nalu_length_size <= item_data.len() {
            let nalu_len = match nalu_length_size {
                1 => item_data[offset] as usize,
                2 => u16::from_be_bytes([item_data[offset], item_data[offset + 1]]) as usize,
                4 => u32::from_be_bytes([
                    item_data[offset],
                    item_data[offset + 1],
                    item_data[offset + 2],
                    item_data[offset + 3],
                ]) as usize,
                _ => 4,
            };
            offset += nalu_length_size;

            if offset + nalu_len > item_data.len() {
                return Err(HeicError::MalformedInput(format!(
                    "NAL unit length {} exceeds remaining slice data at offset {}",
                    nalu_len, offset
                )));
            }

            annex_b.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
            annex_b.extend_from_slice(&item_data[offset..offset + nalu_len]);
            offset += nalu_len;
        }

        Ok(annex_b)
    }

    /// Extracts raw EXIF metadata bytes if present in the container.
    pub fn extract_exif_data(&self, file_bytes: &[u8]) -> HeicResult<Option<Vec<u8>>> {
        let Some(exif_id) = self.exif_item_id else {
            return Ok(None);
        };

        let raw = self.extract_item_data(file_bytes, exif_id)?;
        if raw.len() >= 4 {
            let exif_offset = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]) as usize;
            let start = 4 + exif_offset;
            if start < raw.len() {
                return Ok(Some(raw[start..].to_vec()));
            }
        }

        Ok(Some(raw))
    }
}
