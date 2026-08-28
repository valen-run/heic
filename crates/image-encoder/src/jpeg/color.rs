//! Color conversion from PixelBuffer into planar float YCbCr planes.

use valen_heic_core::PixelFormat;
use valen_image_processing::PixelBuffer;

/// Planar float Y, Cb, Cr image components.
pub struct YCbCrPlanes {
    /// Luma plane `Y`.
    pub y: Vec<f32>,
    /// Chroma blue-difference plane `Cb`.
    pub cb: Vec<f32>,
    /// Chroma red-difference plane `Cr`.
    pub cr: Vec<f32>,
}

/// Converts a pixel buffer into planar float YCbCr buffers shifted by -128.0 for JPEG DCT.
pub fn rgb_to_ycbcr_planes(buffer: &PixelBuffer, width: usize, height: usize) -> YCbCrPlanes {
    let mut y_plane = vec![0.0f32; width * height];
    let mut cb_plane = vec![0.0f32; width * height];
    let mut cr_plane = vec![0.0f32; width * height];

    let bpp = buffer.format.bytes_per_pixel();

    for y in 0..height {
        let row_start = y * buffer.stride;
        for x in 0..width {
            let idx = row_start + x * bpp;
            let (r, g, b) = match buffer.format {
                PixelFormat::Rgb8 => (
                    buffer.data[idx] as f32,
                    buffer.data[idx + 1] as f32,
                    buffer.data[idx + 2] as f32,
                ),
                PixelFormat::Rgba8 => {
                    let a = buffer.data[idx + 3] as f32 / 255.0;
                    (
                        buffer.data[idx] as f32 * a + 255.0 * (1.0 - a),
                        buffer.data[idx + 1] as f32 * a + 255.0 * (1.0 - a),
                        buffer.data[idx + 2] as f32 * a + 255.0 * (1.0 - a),
                    )
                }
                PixelFormat::Bgr8 => (
                    buffer.data[idx + 2] as f32,
                    buffer.data[idx + 1] as f32,
                    buffer.data[idx] as f32,
                ),
                PixelFormat::Bgra8 => {
                    let a = buffer.data[idx + 3] as f32 / 255.0;
                    (
                        buffer.data[idx + 2] as f32 * a + 255.0 * (1.0 - a),
                        buffer.data[idx + 1] as f32 * a + 255.0 * (1.0 - a),
                        buffer.data[idx] as f32 * a + 255.0 * (1.0 - a),
                    )
                }
                _ => (
                    buffer.data[idx] as f32,
                    buffer.data[idx] as f32,
                    buffer.data[idx] as f32,
                ),
            };

            let y_val = 0.299 * r + 0.587 * g + 0.114 * b - 128.0;
            let cb_val = -0.168736 * r - 0.331264 * g + 0.5 * b;
            let cr_val = 0.5 * r - 0.418688 * g - 0.081312 * b;

            y_plane[y * width + x] = y_val;
            cb_plane[y * width + x] = cb_val;
            cr_plane[y * width + x] = cr_val;
        }
    }

    YCbCrPlanes {
        y: y_plane,
        cb: cb_plane,
        cr: cr_plane,
    }
}
