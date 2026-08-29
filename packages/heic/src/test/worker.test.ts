import test from 'node:test';
import assert from 'node:assert/strict';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';
import { handleWorkerRequest } from '../worker/runtime.js';
import { WorkerConverter } from '../worker/client.js';
import { setWasmSource, ensureInitialized } from '../wasm-loader.js';
import { OperationAbortedError, HeicError } from '../errors.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const wasmPath = path.resolve(__dirname, '../../pkg/valen_heic_wasm_bg.wasm');

test('Worker runtime handleWorkerRequest execution', async () => {
  const wasmBuffer = fs.readFileSync(wasmPath);
  setWasmSource(new Uint8Array(wasmBuffer));
  await ensureInitialized();

  // Test detect request
  const validHeic = new Uint8Array([
    0x00, 0x00, 0x00, 0x10,
    0x66, 0x74, 0x79, 0x70,
    0x68, 0x65, 0x69, 0x63,
    0x00, 0x00, 0x00, 0x00,
  ]);

  const detectReq = {
    id: 'req_1',
    type: 'detect' as const,
    data: validHeic.buffer,
  };

  const { response: detectRes } = await handleWorkerRequest(detectReq);
  assert.equal(detectRes.id, 'req_1');
  assert.equal(detectRes.success, true);
  if (detectRes.success && detectRes.payload.type === 'detect') {
    assert.equal(detectRes.payload.result, true);
  }

  // Test abort request
  const abortReq = { id: 'req_2', type: 'abort' as const };
  const { response: abortRes } = await handleWorkerRequest(abortReq);
  assert.equal(abortRes.id, 'req_2');
  assert.equal(abortRes.success, false);
  if (!abortRes.success) {
    assert.equal(abortRes.error.code, 'OPERATION_ABORTED');
  }
});

test('WorkerConverter request multiplexing and response correlation', async () => {
  // Mock Worker implementation for node test environment
  class MockWorker {
    onmessage: ((event: any) => void) | null = null;
    onerror: ((event: any) => void) | null = null;

    postMessage(req: any) {
      setTimeout(async () => {
        const { response } = await handleWorkerRequest(req);
        if (this.onmessage) {
          this.onmessage({ data: response });
        }
      }, 5);
    }

    terminate() {}
  }

  const client = new WorkerConverter(() => new MockWorker() as any);

  const validHeic = new Uint8Array([
    0x00, 0x00, 0x00, 0x10,
    0x66, 0x74, 0x79, 0x70,
    0x68, 0x65, 0x69, 0x63,
    0x00, 0x00, 0x00, 0x00,
  ]);

  // Launch two concurrent requests
  const [res1, res2] = await Promise.all([
    client.detect(validHeic),
    client.detect(validHeic),
  ]);

  assert.equal(res1, true);
  assert.equal(res2, true);

  client.terminate();
});

test('WorkerConverter handles AbortSignal cancellation', async () => {
  class SlowMockWorker {
    onmessage: ((event: any) => void) | null = null;
    onerror: ((event: any) => void) | null = null;

    postMessage() {
      // Intentionally slow
    }

    terminate() {}
  }

  const client = new WorkerConverter(() => new SlowMockWorker() as any);
  const controller = new AbortController();

  const promise = client.send(
    (id) => ({ id, type: 'detect', data: new ArrayBuffer(0) }),
    [],
    controller.signal
  );

  controller.abort();

  await assert.rejects(async () => {
    await promise;
  }, OperationAbortedError);

  client.terminate();
});

test('WorkerConverter auto-recovers after worker crash', async () => {
  let workerInstanceCount = 0;
  let activeWorker: any = null;

  class CrashableWorker {
    onmessage: ((event: any) => void) | null = null;
    onerror: ((event: any) => void) | null = null;

    constructor() {
      workerInstanceCount += 1;
      activeWorker = this;
    }

    postMessage(req: any) {
      setTimeout(async () => {
        const { response } = await handleWorkerRequest(req);
        if (this.onmessage) {
          this.onmessage({ data: response });
        }
      }, 5);
    }

    terminate() {}
  }

  const client = new WorkerConverter(() => new CrashableWorker() as any);

  const validHeic = new Uint8Array([
    0x00, 0x00, 0x00, 0x10,
    0x66, 0x74, 0x79, 0x70,
    0x68, 0x65, 0x69, 0x63,
    0x00, 0x00, 0x00, 0x00,
  ]);

  // 1. Initial request succeeds
  const initialRes = await client.detect(validHeic);
  assert.equal(initialRes, true);
  assert.equal(workerInstanceCount, 1);

  // 2. Simulate worker crash with pending request
  const crashPromise = client.send((id) => ({ id, type: 'inspect', data: new ArrayBuffer(0) }));
  activeWorker.onerror({ message: 'Simulated worker out of memory crash' });

  await assert.rejects(async () => {
    await crashPromise;
  }, HeicError);

  // 3. Subsequent request transparently respawns a fresh worker instance
  const recoveryRes = await client.detect(validHeic);
  assert.equal(recoveryRes, true);
  assert.equal(workerInstanceCount, 2);

  client.terminate();
});
