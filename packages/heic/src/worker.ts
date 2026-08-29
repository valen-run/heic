/**
 * Web Worker integration and entry point for `@valen-run/heic/worker`.
 */

export * from './worker/protocol.js';
export * from './worker/client.js';
export * from './worker/pool.js';
export { initWorkerRuntime, handleWorkerRequest } from './worker/runtime.js';

import { initWorkerRuntime } from './worker/runtime.js';

// Auto-initialize listener if loaded directly inside a Web Worker script context
if (typeof self !== 'undefined' && typeof (self as any).importScripts === 'function') {
  initWorkerRuntime(self);
}
