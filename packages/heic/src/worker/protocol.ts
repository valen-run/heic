/**
 * Web Worker communication protocol and message definitions.
 */

import type { HeicErrorCode } from '../errors.js';
import type { ConvertOptions, DecodeOptions, ProcessingLimits } from '../types/options.js';
import type { DecodedImage, InspectResult } from '../types/results.js';

/**
 * Action types supported by the worker engine.
 */
export type WorkerRequestAction = 'init' | 'detect' | 'inspect' | 'convert' | 'decode' | 'abort';

/**
 * Message payloads dispatched from client to worker.
 */
export type WorkerRequest =
  | { id: string; type: 'init'; wasmSource?: string | ArrayBuffer }
  | { id: string; type: 'detect'; data: ArrayBuffer }
  | { id: string; type: 'inspect'; data: ArrayBuffer; limits?: ProcessingLimits }
  | { id: string; type: 'convert'; data: ArrayBuffer; options?: ConvertOptions }
  | { id: string; type: 'decode'; data: ArrayBuffer; options?: DecodeOptions }
  | { id: string; type: 'abort' };

/**
 * Successful response payloads sent back from worker.
 */
export type WorkerSuccessResult =
  | { type: 'init'; success: true }
  | { type: 'detect'; result: boolean }
  | { type: 'inspect'; result: InspectResult }
  | { type: 'convert'; result: ArrayBuffer }
  | { type: 'decode'; result: DecodedImage };

/**
 * Message responses dispatched from worker back to client.
 */
export type WorkerResponse =
  | { id: string; success: true; payload: WorkerSuccessResult }
  | { id: string; success: false; error: { code: HeicErrorCode; message: string } };
