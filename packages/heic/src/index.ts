/**
 * @valen-run/heic - Production-grade browser HEIC/HEIF processing library.
 */

import { wasmGetRawPixels, wasmIsHeif, wasmIsHeifSync, wasmProbe } from './bridge.js';
import { heicTo } from './converter.js';
import { toUint8Array, toUint8ArraySync } from './input.js';
import type { DecodeOptions, InputSource, ProcessingLimits } from './types/options.js';
import type { DecodedImage, InspectResult } from './types/results.js';

export * from './types.js';
export * from './errors.js';
export { heicTo } from './converter.js';
export { ensureInitialized, setWasmSource, isWasmInitialized } from './wasm-loader.js';

/**
 * Fast asynchronous detection of HEIF/HEIC containers.
 */
export async function isHeic(input: InputSource): Promise<boolean> {
  const bytes = await toUint8Array(input);
  return wasmIsHeif(bytes);
}

/**
 * Alias for [`isHeic`].
 */
export const isHeif = isHeic;

/**
 * Fast synchronous detection for pre-buffered bytes.
 */
export function isHeicSync(input: Uint8Array | ArrayBuffer): boolean {
  const bytes = toUint8ArraySync(input);
  return wasmIsHeifSync(bytes);
}

/**
 * Fast metadata inspection without full bitstream decode.
 */
export async function probe(input: InputSource, limits?: ProcessingLimits): Promise<InspectResult> {
  const bytes = await toUint8Array(input);
  return wasmProbe(bytes, limits);
}

/**
 * Alias for [`probe`].
 */
export const inspect = probe;

/**
 * Decodes a HEIC/HEIF image into an uncompressed raw pixel buffer.
 */
export async function getRawPixels(input: InputSource, options: DecodeOptions = {}): Promise<DecodedImage> {
  const bytes = await toUint8Array(input);
  return wasmGetRawPixels(bytes, options);
}

export default heicTo;
