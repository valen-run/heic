#![no_main]

use libfuzzer_sys::fuzz_target;
use valen_heif_parser::is_heif_or_heic;

fuzz_target!(|data: &[u8]| {
    // Fuzz pure-Rust brand detection function
    let _ = is_heif_or_heic(data);

    // Also fuzz WASM wrapper function
    let _ = valen_heic_wasm::is_heif(data);
});
