# @valen/heic

> **Status: Active Development** — Phase 1 (Pure-Rust Engine) complete and verified. Phase 2 (WebAssembly & TypeScript Browser Adapter) is currently in active progress.

`@valen/heic` is a zero-C-dependency, `#![forbid(unsafe_code)]` HEIC/HEIF decoding and conversion engine written in pure Rust and compiled to WebAssembly, paired with an ergonomic, high-level TypeScript API.

Designed for production web applications requiring fast, client-side HEIC/HEIF detection, metadata extraction, intra-frame decoding, and direct encoding (JPEG, PNG, WebP) with strict resource limits and zero DOM `<canvas>` dependencies.

---

## Development Status & Roadmap

| Phase | Description | Status |
| :--- | :--- | :--- |
| **Phase 1: Rust Core Engine** | Pure-Rust ISO-BMFF parser, HEVC intra decoder, grid assembly, EXIF orientation, JPEG/PNG/WebP encoders, safety limits. | **COMPLETED** (50 tests passing) |
| **Phase 2: WASM & TypeScript** | `wasm-bindgen` interface, TypeScript browser adapter, Web Worker orchestration, streaming WASM loader. | **IN PROGRESS** |
| **Phase 3: QA & CI/CD** | Test corpus, Playwright cross-browser/CSP suite, fuzzing, WASM size optimization, GitHub Actions CI/CD. | **PLANNED** |

---

## Workspace Crates

The Rust core is organized into 5 decoupled, single-responsibility crates adhering to `#![forbid(unsafe_code)]`:

| Crate | Description | Status |
| :--- | :--- | :--- |
| [`valen-heic-core`](crates/heic-core) | Core types (`ImageDimensions`, `PixelFormat`, `Orientation`, `ColorSpace`), structured error models, and defensive resource limits (`Limits`). | Done |
| [`valen-heif-parser`](crates/heif-parser) | Defensive ISO-BMFF / HEIF demuxer (`ftyp`, `meta`, `iloc`, `iinf`, `iprp`, `iref`, `grid`), metadata extraction, and Annex-B stream translation. | Done |
| [`valen-heic-decoder`](crates/heic-decoder) | Pure-Rust HEVC intra bitstream decoder (Exp-Golomb reader, CABAC engine, 35 intra modes, 4x4 DST-VII / DCT-II transforms, deblocking, SAO). | Done |
| [`valen-image-processing`](crates/image-processing) | `PixelBuffer` layout, multi-tile grid assembly with boundary cropping, 8 EXIF orientation transformations, and alpha compositing. | Done |
| [`valen-image-encoder`](crates/image-encoder) | In-WASM target encoders: Baseline Sequential DCT JPEG (with JFIF & byte stuffing), ISO PNG (Deflate/Zlib), and Lossless WebP (VP8L). | Done |
| [`valen-heic-wasm`](wasm) | Safe WebAssembly bindings and zero-copy memory buffer exchange via `wasm-bindgen`. | In Progress |

---

## Implemented Core Capabilities (Phase 1)

- **Pure-Rust Safety**: Entire decoding pipeline is 100% pure safe Rust (`#![forbid(unsafe_code)]`). No C/C++ libraries (no `libde265`, `libheif`), zero memory vulnerabilities.
- **Defensive Resource Limits**: Strict validation against decompression bombs and excessive allocations (`Limits::default_browser()`) on file size, dimensions, pixel count, decoded buffer memory, and tile counts.
- **ISO-BMFF Demuxer & Metadata**: Instant detection (`is_heif_or_heic`) and inspection (`inspect_container`) extracting dimensions, color spaces, EXIF orientation, and alpha masks without full bitstream decoding.
- **HEVC Intra Bitstream Decoding**:
  - Safe bitstream reader with $ue(v)$ and $se(v)$ Exp-Golomb decoding.
  - Full parameter set parsing (`VPS`, `SPS`, `PPS`, `SliceHeader`).
  - Arithmetic CABAC entropy decoder with state renormalization and bypass decoding.
  - 35 Intra Prediction modes (Planar, DC, Angular 2..=34) with 32-step sub-pel interpolation and 3-tap reference smoothing.
  - Inverse quantization with $QP \pmod 6$ scaling factors, 4x4 DST-VII, and partial butterfly DCT-II (4x4, 8x8, 16x16, 32x32).
  - In-loop deblocking filter and Sample Adaptive Offset (SAO).
  - CTU quadtree recursive descent and fixed-point YUV420 to RGB planar reconstruction.
- **Multi-Tile Grid & Transformations**:
  - ISO/IEC 23008-12 multi-tile row-major grid assembly with edge cropping.
  - 8 EXIF orientation transformations (rotations & reflections).
  - Alpha channel compositing and solid background color blending ($C_{out} = \frac{C \cdot \alpha + BG \cdot (255 - \alpha) + 127}{255}$).
- **In-WASM Direct Image Encoders**:
  - **JPEG**: Baseline Sequential DCT with quality scaling (1–100), 2D forward DCT, standard Annex K Huffman coding with `0xFF` byte stuffing, and JFIF header generation.
  - **PNG**: ISO/IEC 15948 compliant encoder with pure-Rust Deflate/Zlib container stream, scanline filtering, and CRC-32 / Adler-32 checksums.
  - **WebP**: Lossless VP8L bitstream encoding inside RIFF containers.

---

## Architecture Overview

```
┌────────────────────────────────────────────────────────┐
│               @valen/heic (TypeScript)                 │
│         API / Worker Orchestration / Type Model        │
└───────────────────────────┬────────────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────┐
│                   wasm/ (wasm-bindgen)                 │
│       Safe WebAssembly Bindings & Error Marshaling     │
└───────────────────────────┬────────────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────┐
│                    Rust Core Workspace                 │
│  ├── heic-core          (types, errors, limits)        │
│  ├── heif-parser        (ISOBMFF box parser, metadata) │
│  ├── heic-decoder       (HEVC intra bitstream decoder) │
│  ├── image-processing   (grid, EXIF orientation, color)│
│  └── image-encoder      (JPEG, PNG, WebP encoders)     │
└────────────────────────────────────────────────────────┘
```

---

## Target TypeScript Usage (Phase 2 Preview)

```typescript
import { detect, inspect, convert, LimitsExceededError } from '@valen/heic';

// 1. Fast container & brand detection
const isHeic = await detect(file);

// 2. Metadata inspection without decoding bitstream
const metadata = await inspect(file);
console.log(`Dimensions: ${metadata.width}x${metadata.height}`);
console.log(`Color Space: ${metadata.colorSpace}, Has Alpha: ${metadata.hasAlpha}`);

// 3. Convert HEIC to JPEG/PNG/WebP with defensive safety limits
const controller = new AbortController();

try {
  const outputBlob = await convert(file, {
    format: 'jpeg',
    quality: 0.85,
    limits: {
      maxFileSize: 50 * 1024 * 1024, // 50 MB
      maxWidth: 8192,
      maxHeight: 8192,
      maxPixelCount: 67_108_864,     // 64 MP
      maxMemoryBytes: 256 * 1024 * 1024,
    },
    signal: controller.signal,
  });
} catch (err) {
  if (err instanceof LimitsExceededError) {
    console.error('Image exceeds allowable dimensions or memory limits', err);
  }
}
```

---

## Development Setup

### Prerequisites
- **Rust Toolchain** (1.75+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **WASM Target**: `rustup target add wasm32-unknown-unknown`
- **Node.js** (v18+) & **pnpm** (v9+)

### Building and Testing

```bash
# Build all Rust workspace crates
cargo build --workspace

# Run all 50+ unit and integration tests
cargo test --workspace

# Run formatting & clippy linters (0 warnings enforced)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

## License

Dual-licensed under either:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
