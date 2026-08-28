//! Parsing raw pixel decode options from JS options.

use super::limits::parse_limits;
use super::reflect::{get_prop_bool, get_prop_string};
use crate::error::invalid_options_error;
use valen_heic_core::{Limits, PixelFormat};
use wasm_bindgen::prelude::*;

/// Options parsed from JavaScript for direct raw pixel decoding.
#[derive(Debug, Clone)]
pub struct WasmDecodeOptions {
    /// Desired pixel format for output buffer.
    pub pixel_format: PixelFormat,
    /// Whether to automatically apply EXIF orientation.
    pub apply_orientation: bool,
    /// Configured safety and resource limits.
    pub limits: Limits,
}

impl Default for WasmDecodeOptions {
    fn default() -> Self {
        Self {
            pixel_format: PixelFormat::Rgba8,
            apply_orientation: true,
            limits: Limits::default(),
        }
    }
}

/// Parses raw pixel decode options from JavaScript [`JsValue`].
pub fn parse_decode_options(options: &JsValue) -> Result<WasmDecodeOptions, JsValue> {
    let limits = parse_limits(options)?;
    let mut config = WasmDecodeOptions {
        limits,
        ..Default::default()
    };

    if options.is_undefined() || options.is_null() {
        return Ok(config);
    }

    // 1. Pixel Format
    if let Some(fmt_str) =
        get_prop_string(options, "pixelFormat").or_else(|| get_prop_string(options, "format"))
    {
        config.pixel_format = match fmt_str.to_lowercase().as_str() {
            "rgba8" | "rgba" => PixelFormat::Rgba8,
            "rgb8" | "rgb" => PixelFormat::Rgb8,
            "bgra8" | "bgra" => PixelFormat::Bgra8,
            "bgr8" | "bgr" => PixelFormat::Bgr8,
            other => {
                return Err(invalid_options_error(&format!(
                    "Unsupported pixel format '{}'. Supported formats: rgba8, rgb8, bgra8, bgr8",
                    other
                )));
            }
        };
    }

    // 2. Orientation
    if let Some(apply_orient) =
        get_prop_bool(options, "applyOrientation").or_else(|| get_prop_bool(options, "autoRotate"))
    {
        config.apply_orientation = apply_orient;
    }

    Ok(config)
}
