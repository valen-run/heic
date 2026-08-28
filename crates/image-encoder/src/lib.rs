//! Image encoding interfaces and format-specific encoder implementations (JPEG, PNG, WebP).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod jpeg;
pub mod png;
pub mod webp;

pub use jpeg::{encode_jpeg, JpegEncoder, JpegOptions};
pub use png::{encode_png, PngEncoder, PngOptions};
pub use webp::{encode_webp, WebpEncoder, WebpOptions};

use valen_heic_core::HeicResult;
use valen_image_processing::PixelBuffer;

/// Generic image encoder interface.
pub trait ImageEncoder {
    /// Encoder specific options type.
    type Options;

    /// Encodes an uncompressed pixel buffer to encoded bytes.
    fn encode(&self, buffer: &PixelBuffer, options: &Self::Options) -> HeicResult<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use valen_heic_core::{ImageDimensions, PixelFormat};

    #[test]
    fn test_image_encoder_traits() {
        let mut buf = PixelBuffer::new(ImageDimensions::new(8, 8), PixelFormat::Rgb8);
        buf.fill(&[200, 100, 50]).unwrap();

        // 1. JPEG trait encode
        let jpeg_enc = JpegEncoder::new();
        let jpeg_bytes = jpeg_enc
            .encode(&buf, &JpegOptions::default())
            .expect("JPEG trait encode should succeed");
        assert!(jpeg_bytes.starts_with(&[0xFF, 0xD8]));

        // 2. PNG trait encode
        let png_enc = PngEncoder::new();
        let png_bytes = png_enc
            .encode(&buf, &PngOptions::default())
            .expect("PNG trait encode should succeed");
        assert!(png_bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]));

        // 3. WebP trait encode
        let webp_enc = WebpEncoder::new();
        let webp_bytes = webp_enc
            .encode(&buf, &WebpOptions::default())
            .expect("WebP trait encode should succeed");
        assert!(webp_bytes.starts_with(b"RIFF"));
    }
}
