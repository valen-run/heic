//! Default safety and resource limit constants.

/// Default maximum input file size for browser environments (256 MiB).
pub const DEFAULT_MAX_FILE_SIZE: u64 = 256 * 1024 * 1024;

/// Default maximum dimension (width or height) in pixels (16,384 px).
pub const DEFAULT_MAX_DIMENSION: u32 = 16_384;

/// Default maximum total pixel count (67,108,864 pixels / 64 Mpx).
pub const DEFAULT_MAX_PIXEL_COUNT: u64 = 64 * 1024 * 1024;

/// Default maximum decoded buffer memory (512 MiB).
pub const DEFAULT_MAX_MEMORY_BYTES: u64 = 512 * 1024 * 1024;

/// Default maximum container items/boxes to parse (defensive anti-DoS limit).
pub const DEFAULT_MAX_ITEM_COUNT: usize = 10_000;

/// Default maximum grid tile count.
pub const DEFAULT_MAX_TILE_COUNT: usize = 1_024;
