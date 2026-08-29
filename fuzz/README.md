# Continuous Fuzzing Setup

This directory contains fuzzing harnesses using [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) and LLVM `libFuzzer` to find panics, buffer overflows, and memory consumption issues on untrusted inputs.

---

## Fuzz Targets

1. **`fuzz_is_heif`**: Fuzzes fast brand detection (`is_heif_or_heic` & WASM `is_heif`).
2. **`fuzz_container_parser`**: Fuzzes ISO-BMFF box parsing, item hierarchy extraction, and metadata inspection (`inspect_container`, `parse_heif`).
3. **`fuzz_decode_with_limits`**: Fuzzes full bitstream decompression, CABAC engine, intra prediction, and image format conversion under safety limits.

---

## Prerequisites

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Ensure Rust nightly toolchain is installed
rustup toolchain install nightly
```

---

## Running Fuzzers Locally

```bash
# Generate seed corpus from fixtures
cargo test --manifest-path fuzz/Cargo.toml

# Run brand detection fuzzer (60 seconds)
cargo +nightly fuzz run fuzz_is_heif -- -max_total_time=60

# Run container parser fuzzer (60 seconds)
cargo +nightly fuzz run fuzz_container_parser -- -max_total_time=60

# Run full decode pipeline fuzzer (60 seconds)
cargo +nightly fuzz run fuzz_decode_with_limits -- -max_total_time=60
```

---

## Sanitizers & Options

`cargo-fuzz` automatically compiles targets with AddressSanitizer (ASan) and UndefinedBehaviorSanitizer (UBSan).

Useful runtime flags:
- `-max_total_time=N`: Maximum execution time in seconds.
- `-max_len=N`: Maximum input buffer length in bytes (default: 4096).
- `-workers=N`: Number of parallel fuzzing workers.
- `-jobs=N`: Number of distinct fuzzing jobs.
