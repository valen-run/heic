//! Standard EXIF orientation transformations (tags 1 through 8).

use crate::error::{HeicError, HeicResult};
use crate::types::dimensions::ImageDimensions;
use std::fmt;

/// Standard EXIF orientation transformations (tags 1 through 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[repr(u8)]
pub enum Orientation {
    /// 1 = Horizontal (normal, top-left)
    #[default]
    Normal = 1,
    /// 2 = Mirror horizontal (top-right)
    MirrorHorizontal = 2,
    /// 3 = Rotate 180 degrees (bottom-right)
    Rotate180 = 3,
    /// 4 = Mirror vertical (bottom-left)
    MirrorVertical = 4,
    /// 5 = Mirror horizontal and rotate 270 CW (left-top)
    MirrorHorizontalRotate270 = 5,
    /// 6 = Rotate 90 CW (right-top)
    Rotate90 = 6,
    /// 7 = Mirror horizontal and rotate 90 CW (right-bottom)
    MirrorHorizontalRotate90 = 7,
    /// 8 = Rotate 270 CW (left-bottom)
    Rotate270 = 8,
}

impl Orientation {
    /// Parses an orientation value from a raw `u8` (1..=8).
    pub fn from_u8(value: u8) -> HeicResult<Self> {
        match value {
            1 => Ok(Self::Normal),
            2 => Ok(Self::MirrorHorizontal),
            3 => Ok(Self::Rotate180),
            4 => Ok(Self::MirrorVertical),
            5 => Ok(Self::MirrorHorizontalRotate270),
            6 => Ok(Self::Rotate90),
            7 => Ok(Self::MirrorHorizontalRotate90),
            8 => Ok(Self::Rotate270),
            other => Err(HeicError::UnsupportedFeature(format!(
                "Invalid or unsupported EXIF orientation tag: {other}"
            ))),
        }
    }

    /// Parses an orientation value from an integer EXIF tag.
    pub fn from_exif(value: u32) -> HeicResult<Self> {
        if value > 255 {
            return Err(HeicError::UnsupportedFeature(format!(
                "Invalid EXIF orientation value: {value}"
            )));
        }
        Self::from_u8(value as u8)
    }

    /// Converts the orientation to its EXIF tag `u8` integer.
    #[inline]
    pub const fn to_u8(&self) -> u8 {
        *self as u8
    }

    /// Returns `true` if applying this orientation swaps the width and height dimensions.
    #[inline]
    pub const fn swaps_dimensions(&self) -> bool {
        matches!(
            self,
            Self::MirrorHorizontalRotate270
                | Self::Rotate90
                | Self::MirrorHorizontalRotate90
                | Self::Rotate270
        )
    }

    /// Computes the transformed dimensions after orientation is applied.
    #[inline]
    pub const fn transform_dimensions(&self, dimensions: ImageDimensions) -> ImageDimensions {
        if self.swaps_dimensions() {
            dimensions.transposed()
        } else {
            dimensions
        }
    }
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Orientation({})", self.to_u8())
    }
}
