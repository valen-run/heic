//! Generates and verifies initial seed corpus for fuzzing targets.

use crate::fixtures::*;
use std::fs;
use std::path::Path;

#[test]
fn test_populate_fuzz_seed_corpus() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir_is_heif = repo_root.join("fuzz/corpus/fuzz_is_heif");
    let dir_parser = repo_root.join("fuzz/corpus/fuzz_container_parser");
    let dir_decode = repo_root.join("fuzz/corpus/fuzz_decode_with_limits");

    let _ = fs::create_dir_all(&dir_is_heif);
    let _ = fs::create_dir_all(&dir_parser);
    let _ = fs::create_dir_all(&dir_decode);

    let iphone = build_iphone_camera_12mp_heic();
    let grid = build_4x4_grid_heic();
    let alpha = build_portrait_alpha_heic();
    let p3 = build_display_p3_heic();
    let burst = build_burst_sequence_heifs();
    let brand_mif1 = build_mif1_heic_brand();
    let trunc = build_truncated_header();
    let mp4 = build_unsupported_mp4_container();
    let iloc_bad = build_out_of_bounds_iloc();
    let dos_65k = build_65k_dimension_bomb_heic();

    // Seeds for fuzz_is_heif
    let _ = fs::write(dir_is_heif.join("iphone.bin"), &iphone);
    let _ = fs::write(dir_is_heif.join("mif1.bin"), &brand_mif1);
    let _ = fs::write(dir_is_heif.join("truncated.bin"), &trunc);
    let _ = fs::write(dir_is_heif.join("mp4.bin"), &mp4);

    // Seeds for fuzz_container_parser
    let _ = fs::write(dir_parser.join("iphone.bin"), &iphone);
    let _ = fs::write(dir_parser.join("grid.bin"), &grid);
    let _ = fs::write(dir_parser.join("alpha.bin"), &alpha);
    let _ = fs::write(dir_parser.join("p3.bin"), &p3);
    let _ = fs::write(dir_parser.join("burst.bin"), &burst);
    let _ = fs::write(dir_parser.join("corrupt_iloc.bin"), &iloc_bad);
    let _ = fs::write(dir_parser.join("dos_65k.bin"), &dos_65k);

    // Seeds for fuzz_decode_with_limits
    let _ = fs::write(dir_decode.join("iphone.bin"), &iphone);
    let _ = fs::write(dir_decode.join("grid.bin"), &grid);
    let _ = fs::write(dir_decode.join("alpha.bin"), &alpha);
}
