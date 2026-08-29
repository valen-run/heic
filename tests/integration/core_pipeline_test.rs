//! Main integration test runner importing modular pipeline test suites.

#[path = "../fixtures/mod.rs"]
pub mod fixtures;

mod test_brand_compatibility;
mod test_color_profile;
mod test_fuzz_seeds;
mod test_grid_assembly;
mod test_iphone_camera;
mod test_orientations;
mod test_portrait_alpha;
