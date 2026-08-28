//! Image item descriptor models.

use crate::boxes::FourCC;
use valen_heic_core::{ColorSpace, ImageDimensions};

/// Descriptor of a single image item inside a HEIF container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageItem {
    /// Item identifier.
    pub id: u32,
    /// Whether this is the primary item of the container.
    pub is_primary: bool,
    /// Item FourCC type (e.g. `hvc1`, `grid`, `Exif`, `mime`).
    pub item_type: FourCC,
    /// Dimensions of the image item.
    pub dimensions: ImageDimensions,
    /// EXIF orientation tag if present.
    pub orientation: Option<u8>,
    /// Color space or profile.
    pub color_space: ColorSpace,
    /// Total data length in bytes.
    pub length: u64,
    /// Whether this item represents an auxiliary alpha mask.
    pub is_alpha_mask: bool,
}
