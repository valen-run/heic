//! Parsing and validation of JavaScript options passed to WASM functions.

pub mod convert;
pub mod decode;
pub mod limits;
pub mod reflect;

pub use convert::{parse_convert_options, WasmConvertOptions};
pub use decode::{parse_decode_options, WasmDecodeOptions};
pub use limits::parse_limits;
pub use reflect::{get_prop_bool, get_prop_f64, get_prop_string};
