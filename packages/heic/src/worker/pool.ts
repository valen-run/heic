/**
 * Shared singleton Worker converter and lifecycle management.
 */

import { WorkerConverter, WorkerFactory } from './client.js';

let sharedConverter: WorkerConverter | null = null;

/**
 * Returns a lazily initialized, shared singleton `WorkerConverter`.
 */
export function getSharedWorkerConverter(factoryOrUrl?: string | URL | WorkerFactory): WorkerConverter {
  if (!sharedConverter) {
    sharedConverter = new WorkerConverter(factoryOrUrl);
  }
  return sharedConverter;
}

/**
 * Terminates and resets the shared singleton WorkerConverter.
 */
export function disposeSharedWorkerConverter(): void {
  if (sharedConverter) {
    sharedConverter.terminate();
    sharedConverter = null;
  }
}
