#![no_main]

use libfuzzer_sys::fuzz_target;
use valen_heic_core::{Limits, OutputFormat};
use valen_heic_wasm::options::WasmConvertOptions;
use valen_heic_wasm::pipeline::{convert_image, decode_to_pixel_buffer};

fuzz_target!(|data: &[u8]| {
    // Restrict fuzzing limits to keep individual iterations fast and prevent OOM
    let limits = Limits::default_browser()
        .with_max_file_size(1024 * 1024)
        .with_max_width(2048)
        .with_max_height(2048)
        .with_max_pixel_count(4_000_000)
        .with_max_memory_bytes(32 * 1024 * 1024);

    // 1. Fuzz decode to pixel buffer
    let _ = decode_to_pixel_buffer(data, &limits, true);

    // 2. Fuzz convert image pipeline (JPEG)
    let convert_opts = WasmConvertOptions {
        format: OutputFormat::Jpeg,
        quality: 80,
        bg_color: [255, 255, 255],
        apply_orientation: true,
        limits,
    };
    let _ = convert_image(data, &convert_opts);
});
