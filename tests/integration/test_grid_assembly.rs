//! Integration tests for 4x4 Grid tiled HEIC containers.

use crate::fixtures::build_4x4_grid_heic;
use valen_heic_core::Limits;
use valen_heif_parser::{inspect_container, parse_heif};

#[test]
fn test_grid_container_inspection() {
    let data = build_4x4_grid_heic();
    let limits = Limits::default();
    let meta = inspect_container(&data, &limits).expect("Inspection should succeed");

    assert!(meta.is_grid);
    assert_eq!(meta.grid_rows, 4);
    assert_eq!(meta.grid_columns, 4);
    assert_eq!(meta.dimensions.width, 2048);
    assert_eq!(meta.dimensions.height, 2048);
}

#[test]
fn test_grid_tile_references_resolution() {
    let data = build_4x4_grid_heic();
    let limits = Limits::default();
    let heif = parse_heif(&data, &limits).expect("Grid parsing should succeed");

    assert!(heif.grid_config.is_some());
    assert_eq!(heif.grid_tile_item_ids.len(), 16);
    assert_eq!(heif.grid_tile_item_ids[0], 2);
    assert_eq!(heif.grid_tile_item_ids[15], 17);
}
