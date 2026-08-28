//! Benchmarks for HEIF container parsing and inspection.

use std::time::Instant;
use valen_heic_core::Limits;
use valen_heif_parser::{inspect_container, is_heif_or_heic};

fn main() {
    let mut header = vec![0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p'];
    header.extend_from_slice(b"heic");
    header.extend_from_slice(&[0, 0, 0, 0]);
    header.extend_from_slice(b"mif1");

    let limits = Limits::none();
    let iterations = 100_000;

    let start = Instant::now();
    for _ in 0..iterations {
        assert!(is_heif_or_heic(&header));
        let _ = inspect_container(&header, &limits);
    }
    let elapsed = start.elapsed();

    println!(
        "Benchmark: {iterations} detections & inspections in {:?} ({:.2} ns/op)",
        elapsed,
        (elapsed.as_nanos() as f64) / (iterations as f64)
    );
}
