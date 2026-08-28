//! Target image output formats and MIME types.

use crate::error::{HeicError, HeicResult};
use std::fmt;

/// Target image formats for conversion and encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputFormat {
    /// JPEG compressed format.
    Jpeg,
    /// PNG lossless compressed format.
    Png,
    /// WebP compressed format.
    WebP,
    /// HEIC / HEIF format.
    Heic,
}

/// Alias for `OutputFormat` for backward compatibility.
pub type ImageFormat = OutputFormat;

impl OutputFormat {
    /// Returns the MIME content type string.
    #[inline]
    pub const fn mime_type(&self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::WebP => "image/webp",
            Self::Heic => "image/heic",
        }
    }

    /// Returns the standard canonical file extension.
    #[inline]
    pub const fn file_extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::Heic => "heic",
        }
    }

    /// Parses an output format from a MIME content-type string.
    pub fn from_mime_type(mime: &str) -> HeicResult<Self> {
        match mime.to_ascii_lowercase().trim() {
            "image/jpeg" | "image/jpg" => Ok(Self::Jpeg),
            "image/png" => Ok(Self::Png),
            "image/webp" => Ok(Self::WebP),
            "image/heic" | "image/heif" => Ok(Self::Heic),
            other => Err(HeicError::UnsupportedFormat(format!(
                "Unsupported output MIME type: {other}"
            ))),
        }
    }

    /// Returns `true` if this output format natively supports an alpha channel.
    #[inline]
    pub const fn supports_alpha(&self) -> bool {
        match self {
            Self::Png | Self::WebP | Self::Heic => true,
            Self::Jpeg => false,
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.mime_type())
    }
}
