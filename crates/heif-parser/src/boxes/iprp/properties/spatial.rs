//! Spatial extents, rotation, and mirror property types.

use valen_heic_core::ImageDimensions;

/// Image spatial extents parsed from `ispe`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageSpatialExtents {
    /// Display width in pixels.
    pub width: u32,
    /// Display height in pixels.
    pub height: u32,
}

impl ImageSpatialExtents {
    /// Converts extents to core [`ImageDimensions`].
    pub const fn to_dimensions(&self) -> ImageDimensions {
        ImageDimensions::new(self.width, self.height)
    }
}

/// Image rotation parsed from `irot`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RotationProperty {
    /// Rotation angle in degrees CCW (0 = 0°, 1 = 90° CCW / 270° CW, 2 = 180°, 3 = 270° CCW / 90° CW).
    pub angle_ccw: u8,
}

impl RotationProperty {
    /// Returns rotation angle in degrees clockwise (0, 90, 180, 270).
    pub const fn angle_cw(&self) -> u16 {
        match self.angle_ccw & 3 {
            0 => 0,
            1 => 270,
            2 => 180,
            3 => 90,
            _ => 0,
        }
    }

    /// Converts to EXIF orientation tag if pure rotation (tag 1, 3, 6, 8).
    pub const fn to_exif_orientation(&self) -> u8 {
        match self.angle_ccw & 3 {
            0 => 1, // Normal
            1 => 8, // 270 CW
            2 => 3, // 180
            3 => 6, // 90 CW
            _ => 1,
        }
    }
}

/// Image mirror parsed from `imir`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MirrorProperty {
    /// 0 = vertical axis (left-right flip), 1 = horizontal axis (top-bottom flip).
    pub axis: u8,
}
