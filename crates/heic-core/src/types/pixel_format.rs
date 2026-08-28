//! Supported pixel buffer formats and channel metadata.

use std::fmt;

/// Supported pixel buffer formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    /// 8-bit per channel RGB (24 bpp).
    Rgb8,
    /// 8-bit per channel RGBA (32 bpp).
    Rgba8,
    /// 10-bit per channel RGB (48 bpp / 16-bit container).
    Rgb10,
    /// 10-bit per channel RGBA (64 bpp / 16-bit container).
    Rgba10,
    /// 8-bit per channel BGR (24 bpp).
    Bgr8,
    /// 8-bit per channel BGRA (32 bpp).
    Bgra8,
}

impl PixelFormat {
    /// Returns the number of color/alpha channels.
    #[inline]
    pub const fn channels(&self) -> usize {
        match self {
            Self::Rgb8 | Self::Rgb10 | Self::Bgr8 => 3,
            Self::Rgba8 | Self::Rgba10 | Self::Bgra8 => 4,
        }
    }

    /// Returns the bit depth per channel (8 or 10).
    #[inline]
    pub const fn bit_depth(&self) -> u8 {
        match self {
            Self::Rgb8 | Self::Rgba8 | Self::Bgr8 | Self::Bgra8 => 8,
            Self::Rgb10 | Self::Rgba10 => 10,
        }
    }

    /// Returns the storage byte size per channel component.
    #[inline]
    pub const fn bytes_per_channel(&self) -> usize {
        match self {
            Self::Rgb8 | Self::Rgba8 | Self::Bgr8 | Self::Bgra8 => 1,
            Self::Rgb10 | Self::Rgba10 => 2,
        }
    }

    /// Returns the number of bytes per pixel in memory.
    #[inline]
    pub const fn bytes_per_pixel(&self) -> usize {
        match self {
            Self::Rgb8 | Self::Bgr8 => 3,
            Self::Rgba8 | Self::Bgra8 => 4,
            Self::Rgb10 => 6,  // 2 bytes per channel * 3
            Self::Rgba10 => 8, // 2 bytes per channel * 4
        }
    }

    /// Returns `true` if the format contains an alpha transparency channel.
    #[inline]
    pub const fn has_alpha(&self) -> bool {
        match self {
            Self::Rgba8 | Self::Rgba10 | Self::Bgra8 => true,
            Self::Rgb8 | Self::Rgb10 | Self::Bgr8 => false,
        }
    }
}

impl fmt::Display for PixelFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rgb8 => write!(f, "RGB8"),
            Self::Rgba8 => write!(f, "RGBA8"),
            Self::Rgb10 => write!(f, "RGB10"),
            Self::Rgba10 => write!(f, "RGBA10"),
            Self::Bgr8 => write!(f, "BGR8"),
            Self::Bgra8 => write!(f, "BGRA8"),
        }
    }
}
