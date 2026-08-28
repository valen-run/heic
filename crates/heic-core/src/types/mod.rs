//! Shared primitive geometry, pixel formats, orientation, and color descriptors.

pub mod color_space;
pub mod dimensions;
pub mod orientation;
pub mod output_format;
pub mod pixel_format;

pub use color_space::ColorSpace;
pub use dimensions::ImageDimensions;
pub use orientation::Orientation;
pub use output_format::{ImageFormat, OutputFormat};
pub use pixel_format::PixelFormat;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_dimensions() {
        let dim = ImageDimensions::new(1920, 1080);
        assert_eq!(dim.width, 1920);
        assert_eq!(dim.height, 1080);
        assert_eq!(dim.pixel_count(), 2_073_600);
        assert_eq!(dim.checked_pixel_count(), Some(2_073_600));
        assert!(!dim.is_empty());
        assert_eq!(format!("{dim}"), "1920x1080");

        let transposed = dim.transposed();
        assert_eq!(transposed.width, 1080);
        assert_eq!(transposed.height, 1920);

        let empty_w = ImageDimensions::new(0, 100);
        assert!(empty_w.is_empty());
        let empty_h = ImageDimensions::new(100, 0);
        assert!(empty_h.is_empty());

        let max_dim = ImageDimensions::new(u32::MAX, u32::MAX);
        assert!(max_dim.checked_pixel_count().is_some());

        // Checked buffer size
        let buf_size = dim.checked_buffer_size(4).unwrap();
        assert_eq!(buf_size, 2_073_600 * 4);
    }

    #[test]
    fn test_pixel_formats() {
        let formats = [
            (PixelFormat::Rgb8, 3, 8, 1, 3, false, "RGB8"),
            (PixelFormat::Rgba8, 4, 8, 1, 4, true, "RGBA8"),
            (PixelFormat::Rgb10, 3, 10, 2, 6, false, "RGB10"),
            (PixelFormat::Rgba10, 4, 10, 2, 8, true, "RGBA10"),
            (PixelFormat::Bgr8, 3, 8, 1, 3, false, "BGR8"),
            (PixelFormat::Bgra8, 4, 8, 1, 4, true, "BGRA8"),
        ];

        for (fmt, channels, depth, bytes_per_ch, bpp, has_alpha, name) in formats {
            assert_eq!(fmt.channels(), channels);
            assert_eq!(fmt.bit_depth(), depth);
            assert_eq!(fmt.bytes_per_channel(), bytes_per_ch);
            assert_eq!(fmt.bytes_per_pixel(), bpp);
            assert_eq!(fmt.has_alpha(), has_alpha);
            assert_eq!(format!("{fmt}"), name);
        }
    }

    #[test]
    fn test_orientation_parsing_and_transforms() {
        for tag in 1..=8 {
            let orient = Orientation::from_u8(tag).unwrap();
            assert_eq!(orient.to_u8(), tag);
            let from_exif = Orientation::from_exif(tag as u32).unwrap();
            assert_eq!(orient, from_exif);
        }

        assert!(Orientation::from_u8(0).is_err());
        assert!(Orientation::from_u8(9).is_err());
        assert!(Orientation::from_exif(1000).is_err());

        let dim = ImageDimensions::new(800, 600);

        // Tags 1..=4 preserve dimensions
        for tag in [1, 2, 3, 4] {
            let o = Orientation::from_u8(tag).unwrap();
            assert!(!o.swaps_dimensions());
            assert_eq!(o.transform_dimensions(dim), dim);
        }

        // Tags 5..=8 swap dimensions
        for tag in [5, 6, 7, 8] {
            let o = Orientation::from_u8(tag).unwrap();
            assert!(o.swaps_dimensions());
            assert_eq!(o.transform_dimensions(dim), ImageDimensions::new(600, 800));
        }
    }

    #[test]
    fn test_output_format() {
        assert_eq!(OutputFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(OutputFormat::Png.mime_type(), "image/png");
        assert_eq!(OutputFormat::WebP.mime_type(), "image/webp");
        assert_eq!(OutputFormat::Heic.mime_type(), "image/heic");

        assert_eq!(OutputFormat::Jpeg.file_extension(), "jpg");
        assert_eq!(OutputFormat::Png.file_extension(), "png");
        assert_eq!(OutputFormat::WebP.file_extension(), "webp");
        assert_eq!(OutputFormat::Heic.file_extension(), "heic");

        assert!(!OutputFormat::Jpeg.supports_alpha());
        assert!(OutputFormat::Png.supports_alpha());
        assert!(OutputFormat::WebP.supports_alpha());

        assert_eq!(
            OutputFormat::from_mime_type("image/jpeg").unwrap(),
            OutputFormat::Jpeg
        );
        assert_eq!(
            OutputFormat::from_mime_type("image/jpg").unwrap(),
            OutputFormat::Jpeg
        );
        assert_eq!(
            OutputFormat::from_mime_type("IMAGE/PNG").unwrap(),
            OutputFormat::Png
        );
        assert_eq!(
            OutputFormat::from_mime_type("image/webp").unwrap(),
            OutputFormat::WebP
        );
        assert_eq!(
            OutputFormat::from_mime_type("image/heic").unwrap(),
            OutputFormat::Heic
        );
        assert!(OutputFormat::from_mime_type("image/gif").is_err());
    }

    #[test]
    fn test_colorspace_default() {
        let cs = ColorSpace::default();
        assert_eq!(cs, ColorSpace::Srgb);
        let p3 = ColorSpace::DisplayP3;
        assert_ne!(cs, p3);
    }
}
