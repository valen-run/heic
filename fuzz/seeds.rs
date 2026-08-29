//! Seed corpus generator using synthetic fixtures.

#[path = "../tests/fixtures/mod.rs"]
pub mod fixtures;

use std::fs;
use std::path::Path;

/// Generates initial seed corpus files for fuzz targets.
pub fn generate_seed_corpora(base_dir: &Path) -> std::io::Result<()> {
    let dir_is_heif = base_dir.join("corpus/fuzz_is_heif");
    let dir_parser = base_dir.join("corpus/fuzz_container_parser");
    let dir_decode = base_dir.join("corpus/fuzz_decode_with_limits");

    fs::create_dir_all(&dir_is_heif)?;
    fs::create_dir_all(&dir_parser)?;
    fs::create_dir_all(&dir_decode)?;

    // Seeds for fuzz_is_heif
    fs::write(dir_is_heif.join("valid_heic.bin"), fixtures::build_iphone_camera_12mp_heic())?;
    fs::write(dir_is_heif.join("valid_mif1.bin"), fixtures::build_mif1_heic_brand())?;
    fs::write(dir_is_heif.join("truncated.bin"), fixtures::build_truncated_header())?;
    fs::write(dir_is_heif.join("unsupported_mp4.bin"), fixtures::build_unsupported_mp4_container())?;

    // Seeds for fuzz_container_parser
    fs::write(dir_parser.join("iphone_12mp.bin"), fixtures::build_iphone_camera_12mp_heic())?;
    fs::write(dir_parser.join("grid_4x4.bin"), fixtures::build_4x4_grid_heic())?;
    fs::write(dir_parser.join("portrait_alpha.bin"), fixtures::build_portrait_alpha_heic())?;
    fs::write(dir_parser.join("display_p3.bin"), fixtures::build_display_p3_heic())?;
    fs::write(dir_parser.join("burst_sequence.bin"), fixtures::build_burst_sequence_heifs())?;
    fs::write(dir_parser.join("corrupt_iloc.bin"), fixtures::build_out_of_bounds_iloc())?;
    fs::write(dir_parser.join("dos_65k.bin"), fixtures::build_65k_dimension_bomb_heic())?;

    // Seeds for fuzz_decode_with_limits
    fs::write(dir_decode.join("iphone_12mp.bin"), fixtures::build_iphone_camera_12mp_heic())?;
    fs::write(dir_decode.join("grid_4x4.bin"), fixtures::build_4x4_grid_heic())?;
    fs::write(dir_decode.join("portrait_alpha.bin"), fixtures::build_portrait_alpha_heic())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_seed_corpora() {
        let fuzz_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        generate_seed_corpora(fuzz_dir).expect("Seed generation should succeed");
    }
}
