# Changelog

All notable changes to `@valen-run/heic` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - Unreleased

### Added
- **Phase 1: Pure-Rust Engine**:
  - `heic-core`: Common types (`ImageDimensions`, `PixelFormat`, `Orientation`, `ColorSpace`), structured error codes, and checked arithmetic bounds (`Limits`).
  - `heif-parser`: Defensive ISO-BMFF box parser (`ftyp`, `meta`, `iloc`, `iinf`, `iprp`, `iref`, `grid`) and Annex-B NAL bitstream extractor.
  - `heic-decoder`: Pure-Rust HEVC intra bitstream decoder with Exp-Golomb reader, CABAC engine, 35 intra modes, 4x4 DST-VII / DCT-II transforms, deblocking, SAO, and quadtree CTU reconstruction.
  - `image-processing`: Multi-tile grid assembly with edge cropping, 8 EXIF orientation rotations/reflections, and alpha channel blending.
  - `image-encoder`: Direct in-WASM image encoders for Baseline Sequential JPEG, ISO PNG (pure Deflate/Zlib), and Lossless WebP (VP8L).
- **Phase 2: WebAssembly Interface & TypeScript Adapter**:
  - `valen-heic-wasm`: Safe `wasm-bindgen` FFI bridge exposing `is_heif`, `probe`, `convert`, and `get_raw_pixels`.
  - `@valen-run/heic`: Modern TypeScript client library with `heicTo` facade, typed errors (`HeicError`, `LimitsExceededError`, `DecodeError`, etc.), and `AbortSignal` cancellation support.
  - `@valen-run/heic/worker`: Resilient Web Worker engine with zero-copy `ArrayBuffer` transfer, request multiplexing by ID, timeout handling, and automatic self-healing crash recovery.
- **CI/CD & Developer Experience**:
  - Continuous integration workflows for Rust formatting, Clippy, unit tests, and TypeScript typechecking.
  - Comprehensive documentation, architectural blueprints, and v2 upgrade roadmap.
