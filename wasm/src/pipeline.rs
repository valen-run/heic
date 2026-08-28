//! In-WASM end-to-end decoding, processing, and encoding pipeline.

use crate::options::{WasmConvertOptions, WasmDecodeOptions};
use valen_heic_core::{HeicResult, Limits, Orientation, OutputFormat};
use valen_heic_decoder::{DecodeOptions, HeicDecoder, PureRustHevcDecoder};
use valen_heif_parser::parse_heif;
use valen_image_encoder::{encode_jpeg, encode_png, encode_webp, PngOptions, WebpOptions};
use valen_image_processing::{
    apply_orientation, assemble_grid, convert_pixel_format, flatten_alpha, merge_alpha_channel,
    PixelBuffer,
};

/// Orchestrates full decoding from HEIF raw bytes into an oriented, alpha-composited [`PixelBuffer`].
pub fn decode_to_pixel_buffer(
    data: &[u8],
    limits: &Limits,
    apply_exif_orientation: bool,
) -> HeicResult<PixelBuffer> {
    limits.check_file_size(data.len() as u64)?;

    // 1. Demux HEIF container
    let heif = parse_heif(data, limits)?;

    let decoder = PureRustHevcDecoder::new();
    let decode_opts = DecodeOptions {
        target_format: None, // Defaults to Rgba8
        apply_orientation: false,
        limits: limits.clone(),
    };

    // 2. Decode primary image (single item or multi-tile grid)
    let mut base_buffer = if let Some(ref grid) = heif.grid_config {
        let mut tiles = Vec::with_capacity(heif.grid_tile_item_ids.len());
        for &tile_id in &heif.grid_tile_item_ids {
            let tile_annex_b = heif.extract_annex_b_stream(data, tile_id)?;
            let tile_buf = decoder.decode_item(&tile_annex_b, &decode_opts)?;
            tiles.push(tile_buf);
        }
        assemble_grid(
            &tiles,
            grid.rows,
            grid.columns,
            grid.output_width,
            grid.output_height,
            limits,
        )?
    } else {
        let annex_b = heif.extract_annex_b_stream(data, heif.primary_item_id)?;
        decoder.decode_item(&annex_b, &decode_opts)?
    };

    // 3. Composite auxiliary alpha plane if present
    if let Some(alpha_id) = heif.alpha_item_id {
        let alpha_annex_b = heif.extract_annex_b_stream(data, alpha_id)?;
        let alpha_buf = decoder.decode_item(&alpha_annex_b, &decode_opts)?;
        base_buffer = merge_alpha_channel(&base_buffer, &alpha_buf, limits)?;
    }

    // 4. Apply EXIF orientation if requested
    if apply_exif_orientation {
        let metadata = heif.get_metadata();
        if let Some(orient_val) = metadata.orientation {
            let orientation = Orientation::from_u8(orient_val)?;
            if orientation != Orientation::Normal {
                base_buffer = apply_orientation(&base_buffer, orientation, limits)?;
            }
        }
    }

    Ok(base_buffer)
}

/// Converts a HEIF/HEIC container into encoded binary bytes (JPEG, PNG, or WebP).
pub fn convert_image(data: &[u8], options: &WasmConvertOptions) -> HeicResult<Vec<u8>> {
    let buffer = decode_to_pixel_buffer(data, &options.limits, options.apply_orientation)?;

    match options.format {
        OutputFormat::Jpeg => {
            // Flatten transparency onto solid background for JPEG
            let rgb_buf = flatten_alpha(&buffer, options.bg_color, &options.limits)?;
            encode_jpeg(&rgb_buf, options.quality)
        }
        OutputFormat::Png => encode_png(&buffer, &PngOptions::default()),
        OutputFormat::WebP => encode_webp(
            &buffer,
            &WebpOptions {
                quality: options.quality as f32,
                lossless: false,
            },
        ),
        OutputFormat::Heic => {
            // For HEIC passthrough or encode, return original data or error
            Ok(data.to_vec())
        }
    }
}

/// Decodes a HEIF/HEIC container to uncompressed raw pixels in requested pixel format.
pub fn decode_raw(data: &[u8], options: &WasmDecodeOptions) -> HeicResult<PixelBuffer> {
    let buffer = decode_to_pixel_buffer(data, &options.limits, options.apply_orientation)?;
    convert_pixel_format(&buffer, options.pixel_format, &options.limits)
}
