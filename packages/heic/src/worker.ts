/**
 * Web Worker integration and message interfaces for offloading image processing.
 */

import { ConvertOptions, DecodeOptions, DecodedImage, InspectResult } from './types.js';

/**
 * Message types dispatched to worker.
 */
export type WorkerRequest =
  | { id: string; type: 'detect'; data: Uint8Array }
  | { id: string; type: 'inspect'; data: Uint8Array }
  | { id: string; type: 'convert'; data: Uint8Array; options?: ConvertOptions }
  | { id: string; type: 'decode'; data: Uint8Array; options?: DecodeOptions };

/**
 * Response types sent back from worker.
 */
export type WorkerResponse =
  | { id: string; success: true; result: boolean | InspectResult | ArrayBuffer | DecodedImage }
  | { id: string; success: false; error: { code: string; message: string } };

/**
 * Sets up a message listener in a Web Worker context.
 */
export function initWorkerHandler(): void {
  if (typeof self !== 'undefined' && 'addEventListener' in self) {
    self.addEventListener('message', (event: MessageEvent<WorkerRequest>) => {
      const { id } = event.data;
      // Worker handler placeholder
      self.postMessage({
        id,
        success: false,
        error: {
          code: 'UNSUPPORTED_FEATURE',
          message: 'Worker handler is in development',
        },
      } satisfies WorkerResponse);
    });
  }
}
