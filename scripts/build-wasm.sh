#!/usr/bin/env bash
set -euo pipefail

# Deterministic WebAssembly compilation & optimization script for @valen-run/heic

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
PKG_DIR="${ROOT_DIR}/packages/heic/pkg"
TARGET_WASM="${ROOT_DIR}/target/wasm32-unknown-unknown/wasm-release/valen_heic_wasm.wasm"

echo "=== [1/4] Building Rust WebAssembly binary (profile: wasm-release) ==="
cargo build \
  --manifest-path "${ROOT_DIR}/Cargo.toml" \
  --package valen-heic-wasm \
  --target wasm32-unknown-unknown \
  --profile wasm-release

echo "=== [2/4] Generating wasm-bindgen JS/TS bindings ==="
mkdir -p "${PKG_DIR}"
wasm-bindgen "${TARGET_WASM}" \
  --out-dir "${PKG_DIR}" \
  --target web \
  --typescript

OUTPUT_WASM="${PKG_DIR}/valen_heic_wasm_bg.wasm"

echo "=== [3/4] Optimizing WASM binary ==="
if command -v wasm-opt &> /dev/null; then
  echo "Running wasm-opt -Oz..."
  wasm-opt -Oz --strip-debug --enable-mutable-globals --enable-bulk-memory "${OUTPUT_WASM}" -o "${OUTPUT_WASM}.opt"
  mv "${OUTPUT_WASM}.opt" "${OUTPUT_WASM}"
else
  echo "wasm-opt not detected in PATH; Rust release profile optimizations (opt-level=z, LTO, strip) active."
fi

echo "=== [4/4] Binary Size Stats ==="
RAW_SIZE=$(wc -c < "${OUTPUT_WASM}")
GZIP_SIZE=$(gzip -9 < "${OUTPUT_WASM}" | wc -c)

echo "Uncompressed WASM: $((RAW_SIZE / 1024)) KB (${RAW_SIZE} bytes)"
echo "Gzipped WASM:      $((GZIP_SIZE / 1024)) KB (${GZIP_SIZE} bytes)"

# Budget check: < 400 KB gzipped
MAX_GZIP_BUDGET=$((400 * 1024))
if [ "${GZIP_SIZE}" -gt "${MAX_GZIP_BUDGET}" ]; then
  echo "ERROR: WASM binary exceeded budget of 400 KB gzipped!" >&2
  exit 1
fi

echo "SUCCESS: WASM binary size is within budget!"
