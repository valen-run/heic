/**
 * Dedicated Web Worker message listener and execution handler.
 */

import { wasmConvert, wasmGetRawPixels, wasmIsHeif, wasmProbe } from '../bridge.js';
import { ensureInitialized, setWasmSource } from '../wasm-loader.js';
import type { WorkerRequest, WorkerResponse } from './protocol.js';

/**
 * Handles a single request message inside the Web Worker.
 */
export async function handleWorkerRequest(req: WorkerRequest): Promise<{ response: WorkerResponse; transfer?: Transferable[] }> {
  const { id, type } = req;

  if (type === 'abort') {
    return {
      response: {
        id,
        success: false,
        error: { code: 'OPERATION_ABORTED', message: 'Operation aborted' },
      },
    };
  }

  try {
    if (type === 'init') {
      if (req.wasmSource) {
        setWasmSource(req.wasmSource as any);
      }
      await ensureInitialized();
      return {
        response: { id, success: true, payload: { type: 'init', success: true } },
      };
    }

    await ensureInitialized();
    const data = new Uint8Array(req.data);

    switch (type) {
      case 'detect': {
        const isHeif = await wasmIsHeif(data);
        return {
          response: { id, success: true, payload: { type: 'detect', result: isHeif } },
        };
      }
      case 'inspect': {
        const metadata = await wasmProbe(data, req.limits);
        return {
          response: { id, success: true, payload: { type: 'inspect', result: metadata } },
        };
      }
      case 'convert': {
        const encoded = await wasmConvert(data, req.options);
        const arrayBuf = (encoded.buffer as ArrayBuffer).slice(
          encoded.byteOffset,
          encoded.byteOffset + encoded.byteLength
        );
        return {
          response: { id, success: true, payload: { type: 'convert', result: arrayBuf } },
          transfer: [arrayBuf],
        };
      }
      case 'decode': {
        const decoded = await wasmGetRawPixels(data, req.options);
        const arrayBuf = (decoded.data.buffer as ArrayBuffer).slice(
          decoded.data.byteOffset,
          decoded.data.byteOffset + decoded.data.byteLength
        );
        const cleanDecoded = {
          ...decoded,
          data: new Uint8Array(arrayBuf),
        };
        return {
          response: { id, success: true, payload: { type: 'decode', result: cleanDecoded } },
          transfer: [arrayBuf],
        };
      }
    }
  } catch (err: any) {
    const code = err?.code || 'INTERNAL_ERROR';
    const message = err?.message || String(err);
    return {
      response: { id, success: false, error: { code, message } },
    };
  }
}

/**
 * Initializes the message listener in the worker global scope.
 */
export function initWorkerRuntime(scope: any = typeof self !== 'undefined' ? self : null): void {
  if (!scope || !scope.addEventListener) {
    return;
  }

  scope.addEventListener('message', async (event: MessageEvent<WorkerRequest>) => {
    const req = event.data;
    if (!req || !req.id) return;

    const { response, transfer } = await handleWorkerRequest(req);
    if (transfer && transfer.length > 0) {
      scope.postMessage(response, transfer);
    } else {
      scope.postMessage(response);
    }
  });
}
