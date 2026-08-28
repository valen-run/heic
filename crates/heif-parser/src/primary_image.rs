//! Primary image identification and item descriptor models.

use valen_heic_core::ImageDimensions;

/// Descriptor of a single image item inside a HEIF container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageItem {
    /// Item identifier.
    pub id: u32,
    /// Whether this is the primary item of the container.
    pub is_primary: bool,
    /// Dimensions of the image item.
    pub dimensions: ImageDimensions,
    /// Data byte offset within the container (or within `mdat`).
    pub offset: u64,
    /// Data length in bytes.
    pub length: u64,
}
