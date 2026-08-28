//! Safe reflection utilities for extracting properties from JS objects.

use wasm_bindgen::prelude::*;

/// Extracts an `f64` number property from a JS object.
pub fn get_prop_f64(obj: &JsValue, key: &str) -> Option<f64> {
    if !obj.is_object() {
        return None;
    }
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_f64())
}

/// Extracts a string property from a JS object.
pub fn get_prop_string(obj: &JsValue, key: &str) -> Option<String> {
    if !obj.is_object() {
        return None;
    }
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_string())
}

/// Extracts a boolean property from a JS object.
pub fn get_prop_bool(obj: &JsValue, key: &str) -> Option<bool> {
    if !obj.is_object() {
        return None;
    }
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_bool())
}
