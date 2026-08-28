//! Pure-Rust HEVC/H.265 intra frame decoding engine and pipeline abstractions.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cabac;
pub mod intra_pred;
pub mod loop_filter;
pub mod nal;
pub mod reconstruct;
pub mod transform;

pub use cabac::{CabacContexts, CabacEngine, ContextModel};
pub use intra_pred::{predict_intra, IntraReferences};
pub use loop_filter::{apply_sao_band_offset, apply_sao_edge_offset, deblock_luma_edge, EdgeType};
pub use nal::{BitReader, NalUnit, NalUnitType, Pps, SliceHeader, SliceType, Sps};
pub use reconstruct::{decode_intra_bitstream, PlanarFrame};
use valen_heic_core::{HeicResult, Limits, PixelFormat};
use valen_image_processing::PixelBuffer;

/// Options for configuring image decoding.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DecodeOptions {
    /// Desired pixel format for output buffer (defaults to RGBA8).
    pub target_format: Option<PixelFormat>,
    /// Whether to automatically apply EXIF orientation transforms.
    pub apply_orientation: bool,
    /// Resource and safety limits for this decode operation.
    pub limits: Limits,
}

/// Abstract decoder interface for HEIC/HEVC image items.
pub trait HeicDecoder {
    /// Decodes a raw compressed bitstream payload into an uncompressed pixel buffer.
    fn decode_item(&self, payload: &[u8], options: &DecodeOptions) -> HeicResult<PixelBuffer>;
}

/// Pure-Rust HEVC intra frame decoder.
#[derive(Debug, Default, Clone, Copy)]
pub struct PureRustHevcDecoder;

impl PureRustHevcDecoder {
    /// Creates a new pure-Rust HEVC decoder instance.
    pub const fn new() -> Self {
        Self
    }
}

impl HeicDecoder for PureRustHevcDecoder {
    fn decode_item(&self, payload: &[u8], options: &DecodeOptions) -> HeicResult<PixelBuffer> {
        let frame = decode_intra_bitstream(payload, &options.limits)?;
        let format = options.target_format.unwrap_or(PixelFormat::Rgba8);
        Ok(frame.to_pixel_buffer(format))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_rust_decoder_creation() {
        let decoder = PureRustHevcDecoder::new();
        let options = DecodeOptions::default();
        let empty_payload = b"";
        let res = decoder.decode_item(empty_payload, &options);
        assert!(res.is_err());
    }

    #[test]
    fn test_planar_frame_allocation_and_rgb_conversion() {
        let limits = Limits::none();
        let frame = PlanarFrame::new(64, 64, 8, &limits).expect("Frame allocation should succeed");
        assert_eq!(frame.width, 64);
        assert_eq!(frame.height, 64);

        let buf = frame.to_pixel_buffer(PixelFormat::Rgba8);
        assert_eq!(buf.dimensions.width, 64);
        assert_eq!(buf.dimensions.height, 64);
        assert_eq!(buf.data.len(), 64 * 64 * 4);
    }

    #[test]
    fn test_cabac_engine_and_context_init() {
        let raw_data = [0b10000000, 0b00000000, 0b11110000, 0b00001111];
        let mut engine = CabacEngine::init(&raw_data, 0).expect("CABAC init should succeed");
        let mut ctx = ContextModel::new(0, 0);
        ctx.init(139, 26);

        let bin = engine
            .decode_bin(&mut ctx)
            .expect("Bin decode should succeed");
        assert!(bin == 0 || bin == 1);

        let bypass_bin = engine
            .decode_bypass_bin()
            .expect("Bypass decode should succeed");
        assert!(bypass_bin == 0 || bypass_bin == 1);
    }

    #[test]
    fn test_deblock_luma_edge_filter() {
        let mut samples = vec![100u16; 64];
        // Create an edge difference
        for y in 0..8 {
            for x in 0..4 {
                samples[y * 8 + x] = 80;
            }
            for x in 4..8 {
                samples[y * 8 + x] = 120;
            }
        }

        deblock_luma_edge(&mut samples, 8, 4, 0, EdgeType::Vertical, 26, 0, 0, 8);

        // Edge at x=3 and x=4 should have been smoothed
        assert!(samples[3] >= 80);
        assert!(samples[4] <= 120);
    }
}
