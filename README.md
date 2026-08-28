# @valen/heic

> **Status: Active Development** — This project is currently under active development. Core architectural interfaces and bindings are established; full HEIC/HEVC decoding is in development and not yet production-ready.

`@valen/heic` is a high-performance, memory-safe browser HEIC/HEIF image processing library written in Rust and compiled to WebAssembly, with an ergonomic TypeScript API.

Designed for robust web applications that require fast, client-side HEIC/HEIF detection, metadata inspection, decoding, and conversion (JPEG, PNG, WebP) with strict resource limits and predictable memory consumption.

---

## Features (Planned & Under Development)

- **Browser-First WebAssembly Engine**: Core parsing and processing compiled to compact, fast WebAssembly.
- **Strict Resource Safety**: Configurable safety limits on file sizes, image dimensions, pixel counts, and memory allocations prior to decoding.
- **Rich Format Detection & Inspection**: Fast metadata inspection (EXIF, color profiles, dimensions) without decoding full bitstreams.
- **Zero Raw Crashes / Safe Error Model**: Structured, typed errors shared cleanly across the WASM boundary.
- **Web Worker Ready**: First-class support for offloading decoding from the main UI thread.
- **Modern JavaScript/TypeScript Support**: Native input handling for `File`, `Blob`, `ArrayBuffer`, and `Uint8Array`.

---

## Intended Usage

```typescript
import { detect, inspect, convert, LimitsExceededError } from '@valen/heic';

// Fast container & brand detection
const isHeic = await detect(file);

// Metadata inspection without decoding the image
const metadata = await inspect(file);
console.log(`Dimensions: ${metadata.width}x${metadata.height}`);

// Convert HEIC to JPEG with safety limits and cancellation support
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
    },
    signal: controller.signal,
  });
} catch (err) {
  if (err instanceof LimitsExceededError) {
    console.error('Image exceeds allowable dimensions or size', err);
  }
}
```

---

## Architecture Overview

The repository is organized as a modular Rust workspace decoupled from the WebAssembly bindings and TypeScript package:

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
│  ├── heic-decoder       (HEVC/HEIC decoding pipeline)  │
│  ├── image-processing   (EXIF orientation, color, buf) │
│  └── image-encoder      (JPEG, PNG, WebP encoders)     │
└────────────────────────────────────────────────────────┘
```

See [docs/architecture.md](docs/architecture.md) for full details.

---

## Development Setup

### Prerequisites
- **Rust toolchain** (1.75+ recommended): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **wasm32 target**: `rustup target add wasm32-unknown-unknown`
- **wasm-pack** (optional for local package packaging): `cargo install wasm-pack`
- **Node.js** (v18+) & **pnpm** (v9+)

### Building and Testing

```bash
# Build all Rust crates
cargo build --workspace

# Run Rust unit and integration tests
cargo test --workspace

# Run Rust formatting & linter checks
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# Run TypeScript type check
pnpm run typecheck
```

---

## Documentation

- [Architecture Guide](docs/architecture.md)
- [Browser Compatibility Matrix](docs/browser-support.md)
- [Security & Resource Safety](docs/security.md)
- [Development & Contribution](docs/development.md)

---

## License

Dual-licensed under either:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
