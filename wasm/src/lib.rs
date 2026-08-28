//! WebAssembly FFI interface and JS bindings for @valen/heic.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use wasm_bindgen::prelude::*;

use valen_heic_core::{HeicError, Limits};
use valen_heif_parser::{inspect_container, is_heif_or_heic};

/// Converts a [`HeicError`] into a [`JsValue`] error with structured code.
fn to_js_error(err: HeicError) -> JsValue {
    let js_err = js_sys::Error::new(&err.to_string());
    let _ = js_sys::Reflect::set(
        &js_err,
        &JsValue::from_str("code"),
        &JsValue::from_str(err.error_code()),
    );
    js_err.into()
}

/// Detects whether the provided byte slice is a supported HEIF/HEIC container.
#[wasm_bindgen]
pub fn wasm_detect(data: &[u8]) -> bool {
    is_heif_or_heic(data)
}

/// Inspects container metadata, enforcing safety limits.
#[wasm_bindgen]
pub fn wasm_inspect(
    data: &[u8],
    max_file_size: Option<f64>,
    max_width: Option<u32>,
    max_height: Option<u32>,
    max_pixel_count: Option<f64>,
) -> Result<JsValue, JsValue> {
    let mut limits = Limits::none();
    if let Some(fs) = max_file_size {
        limits = limits.with_max_file_size(fs as u64);
    }
    if let Some(w) = max_width {
        limits = limits.with_max_width(w);
    }
    if let Some(h) = max_height {
        limits = limits.with_max_height(h);
    }
    if let Some(pc) = max_pixel_count {
        limits = limits.with_max_pixel_count(pc as u64);
    }

    let meta = inspect_container(data, &limits).map_err(to_js_error)?;

    // Build JS Object
    let obj = js_sys::Object::new();
    let major_brand_str = String::from_utf8_lossy(&meta.major_brand).to_string();
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("majorBrand"),
        &JsValue::from_str(&major_brand_str),
    )?;
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("width"),
        &JsValue::from_f64(meta.dimensions.width as f64),
    )?;
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("height"),
        &JsValue::from_f64(meta.dimensions.height as f64),
    )?;
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("imageCount"),
        &JsValue::from_f64(meta.image_count as f64),
    )?;

    Ok(obj.into())
}
