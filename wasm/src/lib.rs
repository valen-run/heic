//! WebAssembly FFI interface and zero-overhead JS bindings for `@valen-run/heic`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod options;
pub mod pipeline;

use wasm_bindgen::prelude::*;

use error::to_js_error;
use options::{parse_convert_options, parse_decode_options, parse_limits};
use pipeline::{convert_image, decode_raw};
use valen_heic_core::{ColorSpace, Limits, PixelFormat};
use valen_heif_parser::{inspect_container, is_heif_or_heic};

/// Fast detection to determine if the provided byte slice is a supported HEIF/HEIC container.
///
/// Inspects the ISO-BMFF `ftyp` box and compatible brand array without allocating image memory.
#[wasm_bindgen]
pub fn is_heif(data: &[u8]) -> bool {
    is_heif_or_heic(data)
}

/// Alias for [`is_heif`] for backward compatibility.
#[wasm_bindgen]
pub fn wasm_detect(data: &[u8]) -> bool {
    is_heif(data)
}

/// Fast metadata inspection without full bitstream decompression.
///
/// Returns dimensions, color space, alpha channel presence, grid properties, and EXIF orientation.
#[wasm_bindgen]
pub fn probe(data: &[u8], options: JsValue) -> Result<JsValue, JsValue> {
    let limits = parse_limits(&options)?;
    let meta = inspect_container(data, &limits).map_err(to_js_error)?;

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
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("hasAlpha"),
        &JsValue::from_bool(meta.has_alpha),
    )?;
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("isGrid"),
        &JsValue::from_bool(meta.is_grid),
    )?;
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("gridRows"),
        &JsValue::from_f64(meta.grid_rows as f64),
    )?;
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("gridColumns"),
        &JsValue::from_f64(meta.grid_columns as f64),
    )?;

    let color_str = match meta.color_space {
        ColorSpace::Srgb => "srgb",
        ColorSpace::DisplayP3 => "display-p3",
        ColorSpace::Rec2020 => "rec2020",
        ColorSpace::Bt601 => "bt601",
        ColorSpace::Bt709 => "bt709",
        ColorSpace::IccProfile(_) => "icc",
    };

    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("colorSpace"),
        &JsValue::from_str(color_str),
    )?;

    if let Some(orient) = meta.orientation {
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("orientation"),
            &JsValue::from_f64(orient as f64),
        )?;
    }

    Ok(obj.into())
}

/// Legacy container inspection endpoint.
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

/// Fully converts a HEIF/HEIC image into encoded binary bytes (JPEG, PNG, or WebP).
///
/// Orchestrates container demuxing, bitstream decompression, grid assembly, alpha compositing,
/// EXIF orientation transformations, and format encoding directly in WebAssembly.
#[wasm_bindgen]
pub fn convert(data: &[u8], options: JsValue) -> Result<js_sys::Uint8Array, JsValue> {
    let opts = parse_convert_options(&options)?;
    let encoded_bytes = convert_image(data, &opts).map_err(to_js_error)?;

    let array = js_sys::Uint8Array::new_with_length(encoded_bytes.len() as u32);
    array.copy_from(&encoded_bytes);
    Ok(array)
}

/// Decodes a HEIF/HEIC image into uncompressed raw pixel data.
///
/// Returns a JavaScript object `{ width, height, format, stride, data: Uint8Array }`.
#[wasm_bindgen]
pub fn get_raw_pixels(data: &[u8], options: JsValue) -> Result<JsValue, JsValue> {
    let opts = parse_decode_options(&options)?;
    let buffer = decode_raw(data, &opts).map_err(to_js_error)?;

    let obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("width"),
        &JsValue::from_f64(buffer.dimensions.width as f64),
    )?;
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("height"),
        &JsValue::from_f64(buffer.dimensions.height as f64),
    )?;
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("format"),
        &JsValue::from_str(match buffer.format {
            PixelFormat::Rgba8 => "rgba8",
            PixelFormat::Rgb8 => "rgb8",
            PixelFormat::Rgb10 => "rgb10",
            PixelFormat::Rgba10 => "rgba10",
            PixelFormat::Bgra8 => "bgra8",
            PixelFormat::Bgr8 => "bgr8",
        }),
    )?;
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("stride"),
        &JsValue::from_f64(buffer.stride as f64),
    )?;

    let array = js_sys::Uint8Array::new_with_length(buffer.data.len() as u32);
    array.copy_from(&buffer.data);

    js_sys::Reflect::set(&obj, &JsValue::from_str("data"), &array.into())?;

    Ok(obj.into())
}
