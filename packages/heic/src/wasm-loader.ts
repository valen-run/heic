/**
 * WebAssembly module loader, streaming instantiator, and caching layer.
 */

import initWasm, { InitInput, InitOutput, initSync } from '../pkg/valen_heic_wasm.js';

let wasmInstance: InitOutput | null = null;
let initPromise: Promise<InitOutput> | null = null;
let customWasmSource: InitInput | null = null;

/**
 * Explicitly sets the WASM binary source URL, ArrayBuffer, or precompiled WebAssembly.Module.
 *
 * Useful for custom bundler setups, CDNs, or offline/Node.js testing.
 */
export function setWasmSource(source: InitInput): void {
  customWasmSource = source;
  wasmInstance = null;
  initPromise = null;
}

/**
 * Returns the active WebAssembly instance, throwing an error if not initialized.
 */
export function getWasmInstance(): InitOutput {
  if (!wasmInstance) {
    throw new Error('WebAssembly module is not initialized. Call ensureInitialized() or an async API function first.');
  }
  return wasmInstance;
}

/**
 * Checks if the WebAssembly module has already finished initialization.
 */
export function isWasmInitialized(): boolean {
  return wasmInstance !== null;
}

/**
 * Resolves the default WASM location relative to this bundle.
 */
function resolveDefaultWasmSource(): InitInput {
  if (customWasmSource) {
    return customWasmSource;
  }

  // Browser / Bundler relative path resolution
  try {
    return new URL('../pkg/valen_heic_wasm_bg.wasm', import.meta.url);
  } catch {
    return 'valen_heic_wasm_bg.wasm';
  }
}

/**
 * Asynchronously initializes and compiles the WebAssembly binary module.
 *
 * Deduplicates concurrent initialization calls.
 */
export async function ensureInitialized(source?: InitInput): Promise<InitOutput> {
  if (wasmInstance) {
    return wasmInstance;
  }

  if (source) {
    customWasmSource = source;
  }

  if (initPromise) {
    return initPromise;
  }

  initPromise = (async () => {
    const wasmSource = resolveDefaultWasmSource();
    try {
      const output = await initWasm({ module_or_path: wasmSource });
      wasmInstance = output;
      return output;
    } catch (err) {
      initPromise = null;
      throw err;
    }
  })();

  return initPromise;
}

/**
 * Synchronously initializes WebAssembly with preloaded binary bytes or Module.
 */
export function ensureInitializedSync(moduleOrBytes: BufferSource | WebAssembly.Module): InitOutput {
  if (wasmInstance) {
    return wasmInstance;
  }
  const output = initSync({ module: moduleOrBytes });
  wasmInstance = output;
  return output;
}
