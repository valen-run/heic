//! Color spaces, auxiliary alpha channel compositing, background blending, and pixel format conversion.

pub mod alpha;
pub mod format;
pub mod profile;

pub use alpha::{flatten_alpha, merge_alpha_channel};
pub use format::convert_pixel_format;
pub use profile::ColorProfileInfo;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pixels::PixelBuffer;
    use valen_heic_core::{ImageDimensions, Limits, PixelFormat};

    #[test]
    fn test_merge_alpha_channel() {
        let limits = Limits::none();
        let dims = ImageDimensions::new(2, 2);

        let mut rgb = PixelBuffer::new(dims, PixelFormat::Rgb8);
        rgb.fill(&[100, 150, 200]).unwrap();

        let mut alpha = PixelBuffer::new(dims, PixelFormat::Rgb8);
        alpha.fill(&[128, 128, 128]).unwrap();

        let rgba = merge_alpha_channel(&rgb, &alpha, &limits).unwrap();
        assert_eq!(rgba.format, PixelFormat::Rgba8);
        assert_eq!(rgba.get_pixel(0, 0), Some(&[100, 150, 200, 128][..]));
    }

    #[test]
    fn test_flatten_alpha_onto_white() {
        let limits = Limits::none();
        let dims = ImageDimensions::new(1, 1);

        let mut rgba = PixelBuffer::new(dims, PixelFormat::Rgba8);
        rgba.set_pixel(0, 0, &[255, 0, 0, 128]).unwrap();

        let rgb = flatten_alpha(&rgba, [255, 255, 255], &limits).unwrap();
        assert_eq!(rgb.format, PixelFormat::Rgb8);
        let px = rgb.get_pixel(0, 0).unwrap();
        assert_eq!(px[0], 255);
        assert!((px[1] as i32 - 127).abs() <= 1);
        assert!((px[2] as i32 - 127).abs() <= 1);
    }

    #[test]
    fn test_convert_pixel_format_rgb_to_bgra() {
        let limits = Limits::none();
        let dims = ImageDimensions::new(1, 1);

        let mut rgb = PixelBuffer::new(dims, PixelFormat::Rgb8);
        rgb.set_pixel(0, 0, &[10, 20, 30]).unwrap();

        let bgra = convert_pixel_format(&rgb, PixelFormat::Bgra8, &limits).unwrap();
        assert_eq!(bgra.format, PixelFormat::Bgra8);
        assert_eq!(bgra.get_pixel(0, 0), Some(&[30, 20, 10, 255][..]));
    }
}
