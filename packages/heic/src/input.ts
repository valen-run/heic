/**
 * Input source normalization, validation, and cancellation checks.
 */

import { OperationAbortedError } from './errors.js';
import type { InputSource } from './types/options.js';

/**
 * Checks if the signal has been aborted and throws an OperationAbortedError if so.
 */
export function checkAborted(signal?: AbortSignal): void {
  if (signal?.aborted) {
    throw new OperationAbortedError();
  }
}

/**
 * Normalizes various browser/Node inputs (Blob, File, ArrayBuffer, Uint8Array) to a Uint8Array.
 */
export async function toUint8Array(input: InputSource): Promise<Uint8Array> {
  if (input instanceof Uint8Array) {
    return input;
  }
  if (input instanceof ArrayBuffer) {
    return new Uint8Array(input);
  }
  if (ArrayBuffer.isView(input)) {
    const view = input as ArrayBufferView;
    return new Uint8Array(view.buffer, view.byteOffset, view.byteLength);
  }
  if (typeof Blob !== 'undefined' && input instanceof Blob) {
    const arrayBuffer = await input.arrayBuffer();
    return new Uint8Array(arrayBuffer);
  }

  throw new TypeError(
    'Invalid input source: expected Blob, File, ArrayBuffer, or Uint8Array.'
  );
}

/**
 * Synchronously normalizes binary array buffers without async promises.
 */
export function toUint8ArraySync(input: Uint8Array | ArrayBuffer | ArrayBufferView): Uint8Array {
  if (input instanceof Uint8Array) {
    return input;
  }
  if (input instanceof ArrayBuffer) {
    return new Uint8Array(input);
  }
  if (ArrayBuffer.isView(input)) {
    const view = input as ArrayBufferView;
    return new Uint8Array(view.buffer, view.byteOffset, view.byteLength);
  }
  throw new TypeError('Invalid synchronous input: expected ArrayBuffer or Uint8Array.');
}
