//! Parsing safety limits from JS options.

use super::reflect::get_prop_f64;
use crate::error::invalid_options_error;
use valen_heic_core::Limits;
use wasm_bindgen::prelude::*;

/// Extracts [`Limits`] from a JavaScript options object.
pub fn parse_limits(options: &JsValue) -> Result<Limits, JsValue> {
    let mut limits = Limits::default();
    if options.is_undefined() || options.is_null() {
        return Ok(limits);
    }

    let limits_obj = if options.is_object() {
        let nested = js_sys::Reflect::get(options, &JsValue::from_str("limits"))
            .unwrap_or(JsValue::UNDEFINED);
        if nested.is_object() {
            nested
        } else {
            options.clone()
        }
    } else {
        options.clone()
    };

    if let Some(fs) = get_prop_f64(&limits_obj, "maxFileSize")
        .or_else(|| get_prop_f64(&limits_obj, "maxInputBytes"))
        .or_else(|| get_prop_f64(options, "maxFileSize"))
        .or_else(|| get_prop_f64(options, "maxInputBytes"))
    {
        if fs < 0.0 || fs > (u64::MAX as f64) {
            return Err(invalid_options_error(
                "maxFileSize must be a non-negative number",
            ));
        }
        limits = limits.with_max_file_size(fs as u64);
    }

    if let Some(w) =
        get_prop_f64(&limits_obj, "maxWidth").or_else(|| get_prop_f64(options, "maxWidth"))
    {
        if w < 0.0 || w > (u32::MAX as f64) {
            return Err(invalid_options_error(
                "maxWidth must be a positive 32-bit integer",
            ));
        }
        limits = limits.with_max_width(w as u32);
    }

    if let Some(h) =
        get_prop_f64(&limits_obj, "maxHeight").or_else(|| get_prop_f64(options, "maxHeight"))
    {
        if h < 0.0 || h > (u32::MAX as f64) {
            return Err(invalid_options_error(
                "maxHeight must be a positive 32-bit integer",
            ));
        }
        limits = limits.with_max_height(h as u32);
    }

    if let Some(pc) = get_prop_f64(&limits_obj, "maxPixelCount")
        .or_else(|| get_prop_f64(&limits_obj, "maxPixels"))
        .or_else(|| get_prop_f64(options, "maxPixelCount"))
        .or_else(|| get_prop_f64(options, "maxPixels"))
    {
        if pc < 0.0 || pc > (u64::MAX as f64) {
            return Err(invalid_options_error(
                "maxPixelCount must be a non-negative number",
            ));
        }
        limits = limits.with_max_pixel_count(pc as u64);
    }

    if let Some(mb) = get_prop_f64(&limits_obj, "maxMemoryBytes")
        .or_else(|| get_prop_f64(options, "maxMemoryBytes"))
    {
        if mb < 0.0 || mb > (u64::MAX as f64) {
            return Err(invalid_options_error(
                "maxMemoryBytes must be a non-negative number",
            ));
        }
        limits = limits.with_max_memory_bytes(mb as u64);
    }

    Ok(limits)
}
