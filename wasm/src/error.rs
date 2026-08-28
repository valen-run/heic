//! Error conversion and structured JS exception generation for `@valen-run/heic`.

use valen_heic_core::HeicError;
use wasm_bindgen::prelude::*;

/// Converts a [`HeicError`] into a JavaScript [`js_sys::Error`] with a structured `code` property.
pub fn to_js_error(err: HeicError) -> JsValue {
    let js_err = js_sys::Error::new(&err.to_string());
    let _ = js_sys::Reflect::set(
        &js_err,
        &JsValue::from_str("code"),
        &JsValue::from_str(err.error_code()),
    );
    let _ = js_sys::Reflect::set(
        &js_err,
        &JsValue::from_str("name"),
        &JsValue::from_str("HeicError"),
    );
    js_err.into()
}

/// Creates a JavaScript [`js_sys::Error`] for invalid options with `INVALID_OPTIONS` code.
pub fn invalid_options_error(message: &str) -> JsValue {
    let js_err = js_sys::Error::new(message);
    let _ = js_sys::Reflect::set(
        &js_err,
        &JsValue::from_str("code"),
        &JsValue::from_str("INVALID_OPTIONS"),
    );
    let _ = js_sys::Reflect::set(
        &js_err,
        &JsValue::from_str("name"),
        &JsValue::from_str("HeicError"),
    );
    js_err.into()
}
