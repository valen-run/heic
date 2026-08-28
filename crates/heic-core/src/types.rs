//! Shared primitive geometry, pixel formats, and image descriptors.

/// 2D image dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ImageDimensions {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl ImageDimensions {
    /// Constructs new image dimensions.
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Computes total pixel count (`width * height`).
    pub const fn pixel_count(&self) -> u64 {
        (self.width as u64) * (self.height as u64)
    }
}

/// Supported pixel buffer formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// 8-bit per channel RGB.
    Rgb8,
    /// 8-bit per channel RGBA.
    Rgba8,
    /// 10-bit per channel RGB.
    Rgb10,
    /// 10-bit per channel RGBA.
    Rgba10,
}

impl PixelFormat {
    /// Returns the number of bytes per pixel.
    pub const fn bytes_per_pixel(&self) -> usize {
        match self {
            Self::Rgb8 => 3,
            Self::Rgba8 => 4,
            Self::Rgb10 => 6,  // 2 bytes per channel * 3
            Self::Rgba10 => 8, // 2 bytes per channel * 4
        }
    }
}

/// Target image formats for conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// HEIC / HEIF format.
    Heic,
    /// JPEG format.
    Jpeg,
    /// PNG format.
    Png,
    /// WebP format.
    WebP,
}

/// Color space identification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorSpace {
    /// Standard sRGB.
    #[default]
    Srgb,
    /// Display P3 wide gamut.
    DisplayP3,
    /// Rec. 2020 wide gamut.
    Rec2020,
    /// Raw embedded ICC profile present.
    IccProfile,
}
