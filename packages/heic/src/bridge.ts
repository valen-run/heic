/**
 * Typed bridge for invoking WebAssembly FFI functions with error mapping.
 */

import * as wasmPkg from '../pkg/valen_heic_wasm.js';
import { fromWasmError } from './errors.js';
import type { ConvertOptions, DecodeOptions, ProcessingLimits } from './types/options.js';
import type { DecodedImage, InspectResult } from './types/results.js';
import { ensureInitialized, isWasmInitialized } from './wasm-loader.js';

/**
 * Fast synchronous check using WASM if initialized, or fallback to pure byte inspection.
 */
export function wasmIsHeifSync(data: Uint8Array): boolean {
  if (isWasmInitialized()) {
    try {
      return wasmPkg.is_heif(data);
    } catch {
      return false;
    }
  }

  // Pure byte inspection fallback
  if (data.length < 12) return false;
  const isFtyp = data[4] === 0x66 && data[5] === 0x74 && data[6] === 0x79 && data[7] === 0x70;
  if (!isFtyp) return false;
  const brand = String.fromCharCode(data[8], data[9], data[10], data[11]).toLowerCase();
  const supported = ['heic', 'heix', 'hevc', 'hevx', 'mif1', 'msf1', 'avic', 'heis'];
  return supported.includes(brand);
}

/**
 * Fast asynchronous HEIF container check.
 */
export async function wasmIsHeif(data: Uint8Array): Promise<boolean> {
  await ensureInitialized();
  try {
    return wasmPkg.is_heif(data);
  } catch {
    return false;
  }
}

/**
 * Container metadata inspection without full bitstream decode.
 */
export async function wasmProbe(data: Uint8Array, limits?: ProcessingLimits): Promise<InspectResult> {
  await ensureInitialized();
  try {
    const raw = wasmPkg.probe(data, limits ? { limits } : {});
    return raw as InspectResult;
  } catch (err) {
    throw fromWasmError(err);
  }
}

/**
 * Executes full bitstream decode + orientation + encoding pipeline in WASM.
 */
export async function wasmConvert(data: Uint8Array, options: ConvertOptions = {}): Promise<Uint8Array> {
  await ensureInitialized();
  try {
    return wasmPkg.convert(data, options);
  } catch (err) {
    throw fromWasmError(err);
  }
}

/**
 * Decodes raw uncompressed pixel buffer from bitstream.
 */
export async function wasmGetRawPixels(data: Uint8Array, options: DecodeOptions = {}): Promise<DecodedImage> {
  await ensureInitialized();
  try {
    const res = wasmPkg.get_raw_pixels(data, options);
    return res as DecodedImage;
  } catch (err) {
    throw fromWasmError(err);
  }
}
