# Security & Resource Safety

Processing untrusted image files uploaded by users presents significant security challenges, including decompression bombs, allocation panics, and parsing vulnerabilities.

`@valen/heic` is built with a defensive security posture:

## 1. WebAssembly Sandbox Isolation

- The Rust core executes entirely inside the browser's WebAssembly sandbox.
- WASM does not have direct access to the host DOM, filesystem, or network.

## 2. Resource Limits & Pre-Decode Validation

Before allocating memory for pixel decoding, the parser extracts image header descriptors (`ispe` boxes) and checks against user-configured `Limits`:

- **Max Input File Size**: Rejects oversized inputs before processing.
- **Max Width & Height**: Prevents pathological dimension values (e.g. 65536x65536).
- **Max Pixel Count**: Guards against decompression bombs.
- **Memory Caps**: Ensures internal buffers do not trigger out-of-memory crashes in memory-constrained devices.

## 3. Panic-Free & Safe Rust

- Crates use `#![forbid(unsafe_code)]` wherever possible.
- Box and container parsing relies on safe slice operations and checked bounds to prevent out-of-bounds reads.
- Errors are returned as `Result<T, HeicError>` instead of panicking.

## 4. Fuzzing & Regression Testing

- The `tests/fixtures/` and `tests/regression/` test suites contain anonymized test cases to catch edge cases, truncated streams, and malformed container structures.
