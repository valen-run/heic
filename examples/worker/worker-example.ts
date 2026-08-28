/**
 * Web Worker example for @valen/heic offloading heavy processing.
 */

import { WorkerRequest, WorkerResponse } from '@valen/heic/worker';

export function createHeicWorkerPool() {
  const worker = new Worker(new URL('@valen/heic/worker', import.meta.url), {
    type: 'module',
  });

  return {
    async inspectInWorker(data: Uint8Array): Promise<unknown> {
      return new Promise((resolve, reject) => {
        const id = Math.random().toString(36).substring(2, 9);

        const handler = (event: MessageEvent<WorkerResponse>) => {
          if (event.data.id === id) {
            worker.removeEventListener('message', handler);
            if (event.data.success) {
              resolve(event.data.result);
            } else {
              reject(new Error(event.data.error.message));
            }
          }
        };

        worker.addEventListener('message', handler);
        const req: WorkerRequest = { id, type: 'inspect', data };
        worker.postMessage(req, [data.buffer]);
      });
    },
    terminate() {
      worker.terminate();
    },
  };
}
