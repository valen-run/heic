//! Main regression test runner importing modular regression and security test suites.

#[path = "../fixtures/mod.rs"]
pub mod fixtures;

mod test_burst_sequence;
mod test_corrupted_boxes;
mod test_dimension_bomb;
mod test_truncated_nal;
