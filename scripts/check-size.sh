#!/usr/bin/env bash
set -euo pipefail

# Automated size verification script for @valen-run/heic artifacts

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
WASM_FILE="${ROOT_DIR}/packages/heic/pkg/valen_heic_wasm_bg.wasm"

if [ ! -f "${WASM_FILE}" ]; then
  echo "ERROR: WASM binary not found at ${WASM_FILE}. Run 'pnpm build:wasm' first." >&2
  exit 1
fi

RAW_SIZE=$(wc -c < "${WASM_FILE}")
GZIP_SIZE=$(gzip -9 < "${WASM_FILE}" | wc -c)

MAX_RAW_BUDGET=$((1200 * 1024))  # 1.2 MB uncompressed budget
MAX_GZIP_BUDGET=$((400 * 1024))   # 400 KB gzipped budget

echo "----------------------------------------"
echo " @valen-run/heic Size Verification"
echo "----------------------------------------"
echo " Uncompressed: $((RAW_SIZE / 1024)) KB (budget: 1200 KB)"
echo " Gzipped:      $((GZIP_SIZE / 1024)) KB (budget: 400 KB)"
echo "----------------------------------------"

if [ "${RAW_SIZE}" -gt "${MAX_RAW_BUDGET}" ]; then
  echo "FAIL: Uncompressed WASM size (${RAW_SIZE} bytes) exceeded budget (${MAX_RAW_BUDGET} bytes)" >&2
  exit 1
fi

if [ "${GZIP_SIZE}" -gt "${MAX_GZIP_BUDGET}" ]; then
  echo "FAIL: Gzipped WASM size (${GZIP_SIZE} bytes) exceeded budget (${MAX_GZIP_BUDGET} bytes)" >&2
  exit 1
fi

echo "PASS: All binary size gates met!"
