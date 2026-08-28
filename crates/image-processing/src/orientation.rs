//! EXIF orientation transformations (rotations and flips).

use valen_heic_core::{HeicError, HeicResult, ImageDimensions};

/// Standard EXIF orientation tags (1 through 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ExifOrientation {
    /// 1 = Horizontal (normal)
    #[default]
    Normal = 1,
    /// 2 = Mirror horizontal
    MirrorHorizontal = 2,
    /// 3 = Rotate 180
    Rotate180 = 3,
    /// 4 = Mirror vertical
    MirrorVertical = 4,
    /// 5 = Mirror horizontal and rotate 270 CW
    MirrorHorizontalRotate270Cw = 5,
    /// 6 = Rotate 90 CW
    Rotate90Cw = 6,
    /// 7 = Mirror horizontal and rotate 90 CW
    MirrorHorizontalRotate90Cw = 7,
    /// 8 = Rotate 270 CW
    Rotate270Cw = 8,
}

impl ExifOrientation {
    /// Parses an EXIF orientation value.
    pub fn from_u8(value: u8) -> HeicResult<Self> {
        match value {
            1 => Ok(Self::Normal),
            2 => Ok(Self::MirrorHorizontal),
            3 => Ok(Self::Rotate180),
            4 => Ok(Self::MirrorVertical),
            5 => Ok(Self::MirrorHorizontalRotate270Cw),
            6 => Ok(Self::Rotate90Cw),
            7 => Ok(Self::MirrorHorizontalRotate90Cw),
            8 => Ok(Self::Rotate270Cw),
            other => Err(HeicError::UnsupportedFeature(format!(
                "Invalid or unsupported EXIF orientation: {other}"
            ))),
        }
    }

    /// Computes the transformed dimensions after orientation is applied.
    pub fn transform_dimensions(&self, dimensions: ImageDimensions) -> ImageDimensions {
        match self {
            Self::Normal | Self::MirrorHorizontal | Self::Rotate180 | Self::MirrorVertical => {
                dimensions
            }
            Self::MirrorHorizontalRotate270Cw
            | Self::Rotate90Cw
            | Self::MirrorHorizontalRotate90Cw
            | Self::Rotate270Cw => ImageDimensions {
                width: dimensions.height,
                height: dimensions.width,
            },
        }
    }
}
