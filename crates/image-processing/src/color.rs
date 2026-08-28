//! Color spaces and color conversion stubs.

use valen_heic_core::ColorSpace;

/// Color profile and color transfer description.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ColorProfileInfo {
    /// Identified color space.
    pub space: ColorSpace,
    /// Raw ICC profile data if embedded in the container.
    pub raw_icc: Option<Vec<u8>>,
}
