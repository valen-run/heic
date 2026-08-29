# @valen-run/heic

> **Status: Active Development** — Phase 1 (Pure-Rust Engine) & Phase 2 (WebAssembly & TypeScript Browser Adapter) completed and verified. Phase 3 (Quality Assurance, Tooling & CI/CD) is in progress.

`@valen-run/heic` is a zero-C-dependency, `#![forbid(unsafe_code)]` HEIC/HEIF decoding and conversion engine written in pure Rust and compiled to WebAssembly, paired with an ergonomic, high-level TypeScript API and multi-threaded Web Worker architecture.

Designed for production web applications requiring fast, client-side HEIC/HEIF detection, metadata extraction, intra-frame decoding, and direct encoding (JPEG, PNG, WebP) with strict resource limits and zero DOM `<canvas>` dependencies.

---

## Development Status & Roadmap

| Phase | Description | Status |
| :--- | :--- | :--- |
| **Phase 1: Rust Core Engine** | Pure-Rust ISO-BMFF parser, HEVC intra decoder, grid assembly, EXIF orientation, JPEG/PNG/WebP encoders, safety limits. | **COMPLETED** (50+ tests passing) |
| **Phase 2: WASM & TypeScript** | `wasm-bindgen` interface, TypeScript browser adapter (`heicTo`), Web Worker client with request multiplexing and auto-recovery, streaming WASM loader. | **COMPLETED** (15 TS tests + 63 Rust tests passing) |
| **Phase 3: QA & CI/CD** | Test corpus, Playwright cross-browser/CSP suite, fuzzing, WASM size optimization, GitHub Actions CI/CD. | **IN PROGRESS** |

---

## Workspace Crates & Packages

The engine is organized into decoupled, single-responsibility crates adhering to `#![forbid(unsafe_code)]` and modular TypeScript packages:

| Crate / Package | Description | Status |
| :--- | :--- | :--- |
| [`crates/heic-core`](crates/heic-core) | Core types (`ImageDimensions`, `PixelFormat`, `Orientation`, `ColorSpace`), structured error models, and defensive resource limits (`Limits`). | Done |
| [`crates/heif-parser`](crates/heif-parser) | Defensive ISO-BMFF / HEIF demuxer (`ftyp`, `meta`, `iloc`, `iinf`, `iprp`, `iref`, `grid`), metadata extraction, and Annex-B stream translation. | Done |
| [`crates/heic-decoder`](crates/heic-decoder) | Pure-Rust HEVC intra bitstream decoder (Exp-Golomb reader, CABAC engine, 35 intra modes, 4x4 DST-VII / DCT-II transforms, deblocking, SAO). | Done |
| [`crates/image-processing`](crates/image-processing) | `PixelBuffer` layout, multi-tile grid assembly with boundary cropping, 8 EXIF orientation transformations, and alpha compositing. | Done |
| [`crates/image-encoder`](crates/image-encoder) | In-WASM target encoders: Baseline Sequential DCT JPEG (with JFIF & byte stuffing), ISO PNG (Deflate/Zlib), and Lossless WebP (VP8L). | Done |
| [`wasm`](wasm) | Safe WebAssembly bindings and zero-copy memory buffer exchange via `wasm-bindgen`. | Done |
| [`packages/heic`](packages/heic) | Production-grade TypeScript SDK (`@valen-run/heic`) with `heicTo` facade, typed errors, and resilient Web Worker client (`@valen-run/heic/worker`). | Done |

---

## Implemented Core Capabilities

- **Pure-Rust Safety**: Entire decoding pipeline is 100% pure safe Rust (`#![forbid(unsafe_code)]`). No C/C++ libraries (no `libde265`, `libheif`), zero memory vulnerabilities.
- **Defensive Resource Limits**: Strict validation against decompression bombs and excessive allocations (`Limits::default_browser()`) on file size, dimensions, pixel count, decoded buffer memory, and tile counts.
- **ISO-BMFF Demuxer & Metadata**: Instant detection (`isHeic` / `isHeicSync`) and inspection (`probe` / `inspect`) extracting dimensions, color spaces, EXIF orientation, and alpha masks without full bitstream decoding.
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
- **Resilient Web Worker Engine**:
  - Off-thread execution preventing UI freezes during large batch operations.
  - Zero-copy `ArrayBuffer` transfers on input and output.
  - Request multiplexing, timeout management, `AbortSignal` cancellation, and self-healing auto-recovery upon worker crash.

---

## TypeScript Usage Guide

### 1. Main Thread Conversion (`@valen-run/heic`)

```typescript
import heicTo, { isHeic, probe, LimitsExceededError } from '@valen-run/heic';

// 1. Fast container & brand detection
const isSupported = await isHeic(file);

// 2. Metadata inspection without full bitstream decompression
const metadata = await probe(file);
console.log(`Dimensions: ${metadata.width}x${metadata.height}`);
console.log(`Color Space: ${metadata.colorSpace}, Has Alpha: ${metadata.hasAlpha}`);

// 3. Convert HEIC to JPEG / PNG / WebP with defensive limits & cancellation
const controller = new AbortController();

try {
  const outputBlob = await heicTo(file, {
    type: 'image/jpeg',
    quality: 0.85,
    applyOrientation: true,
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

### 2. Multi-Threaded Web Worker (`@valen-run/heic/worker`)

```typescript
import { getSharedWorkerConverter, WorkerConverter } from '@valen-run/heic/worker';

// Use the shared singleton worker instance
const worker = getSharedWorkerConverter();

// Zero-copy conversion off the main UI thread
const jpegBlob = await worker.convert(file, {
  format: 'jpeg',
  quality: 90,
});
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

# Run all 63 unit and integration tests
cargo test --workspace

# Run formatting & clippy linters (0 warnings enforced)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# Build TypeScript package & run test suite
node ./packages/heic/node_modules/typescript/bin/tsc --project packages/heic/tsconfig.json
node --test packages/heic/dist/test/*.js
```

---

## License

Dual-licensed under either:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
