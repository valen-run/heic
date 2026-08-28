//! Parsing full conversion options from JS options.

use super::limits::parse_limits;
use super::reflect::{get_prop_bool, get_prop_f64, get_prop_string};
use crate::error::invalid_options_error;
use valen_heic_core::{Limits, OutputFormat};
use wasm_bindgen::prelude::*;

/// Options parsed from JavaScript for full image conversion.
#[derive(Debug, Clone)]
pub struct WasmConvertOptions {
    /// Target image output format (JPEG, PNG, WebP).
    pub format: OutputFormat,
    /// Encoding quality between 1 and 100.
    pub quality: u8,
    /// Background RGB color for alpha flattening (e.g. for JPEG).
    pub bg_color: [u8; 3],
    /// Whether to automatically apply EXIF orientation.
    pub apply_orientation: bool,
    /// Configured safety and resource limits.
    pub limits: Limits,
}

impl Default for WasmConvertOptions {
    fn default() -> Self {
        Self {
            format: OutputFormat::Jpeg,
            quality: 92,
            bg_color: [255, 255, 255],
            apply_orientation: true,
            limits: Limits::default(),
        }
    }
}

/// Parses full conversion options from JavaScript [`JsValue`].
pub fn parse_convert_options(options: &JsValue) -> Result<WasmConvertOptions, JsValue> {
    let limits = parse_limits(options)?;
    let mut config = WasmConvertOptions {
        limits,
        ..Default::default()
    };

    if options.is_undefined() || options.is_null() {
        return Ok(config);
    }

    // 1. Output Format
    if let Some(fmt_str) =
        get_prop_string(options, "format").or_else(|| get_prop_string(options, "type"))
    {
        config.format = match fmt_str.to_lowercase().as_str() {
            "jpeg" | "jpg" | "image/jpeg" | "image/jpg" => OutputFormat::Jpeg,
            "png" | "image/png" => OutputFormat::Png,
            "webp" | "image/webp" => OutputFormat::WebP,
            other => {
                return Err(invalid_options_error(&format!(
                    "Unsupported output format '{}'. Supported formats: jpeg, png, webp",
                    other
                )));
            }
        };
    }

    // 2. Quality
    if let Some(q) = get_prop_f64(options, "quality") {
        if !(0.0..=1.0).contains(&q) && !(1.0..=100.0).contains(&q) {
            return Err(invalid_options_error(
                "quality must be a number between 0.0 and 1.0 (or 1 to 100)",
            ));
        }

        config.quality = if q <= 1.0 {
            ((q * 100.0).round() as u8).clamp(1, 100)
        } else {
            (q.round() as u8).clamp(1, 100)
        };
    }

    // 3. Orientation
    if let Some(apply_orient) =
        get_prop_bool(options, "applyOrientation").or_else(|| get_prop_bool(options, "autoRotate"))
    {
        config.apply_orientation = apply_orient;
    }

    // 4. Background Color
    if options.is_object() {
        if let Ok(bg_val) = js_sys::Reflect::get(options, &JsValue::from_str("backgroundColor"))
            .or_else(|_| js_sys::Reflect::get(options, &JsValue::from_str("background")))
        {
            if js_sys::Array::is_array(&bg_val) {
                let arr = js_sys::Array::from(&bg_val);
                if arr.length() >= 3 {
                    let r = arr.get(0).as_f64().unwrap_or(255.0).clamp(0.0, 255.0) as u8;
                    let g = arr.get(1).as_f64().unwrap_or(255.0).clamp(0.0, 255.0) as u8;
                    let b = arr.get(2).as_f64().unwrap_or(255.0).clamp(0.0, 255.0) as u8;
                    config.bg_color = [r, g, b];
                }
            }
        }
    }

    Ok(config)
}
