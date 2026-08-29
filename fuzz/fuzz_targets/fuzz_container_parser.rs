#![no_main]

use libfuzzer_sys::fuzz_target;
use valen_heic_core::Limits;
use valen_heif_parser::{inspect_container, parse_heif};

fuzz_target!(|data: &[u8]| {
    let limits = Limits::default_browser()
        .with_max_file_size(10 * 1024 * 1024)
        .with_max_width(4096)
        .with_max_height(4096)
        .with_max_pixel_count(16_000_000);

    // Fuzz container inspection
    let _ = inspect_container(data, &limits);

    // Fuzz full container demuxing & item hierarchy resolution
    let _ = parse_heif(data, &limits);
});
