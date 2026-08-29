/**
 * Resilient Web Worker client with multiplexing, zero-copy buffer transfer, and auto-restart.
 */

import { fromWasmError, HeicError } from '../errors.js';
import { checkAborted, toUint8Array } from '../input.js';
import type { ConvertOptions, DecodeOptions, InputSource, ProcessingLimits } from '../types/options.js';
import type { DecodedImage, InspectResult } from '../types/results.js';
import { PendingRequestTracker } from './pending.js';
import type { WorkerRequest, WorkerResponse } from './protocol.js';

export type WorkerFactory = () => Worker;

/**
 * Client for managing off-thread Web Worker HEIC conversion jobs.
 */
export class WorkerConverter {
  private worker: Worker | null = null;
  private state: 'UNINITIALIZED' | 'READY' | 'FAILED' | 'DISPOSED' = 'UNINITIALIZED';
  private tracker = new PendingRequestTracker();
  private requestCounter = 0;
  private readonly workerFactory: WorkerFactory;
  private readonly timeoutMs: number;

  constructor(factoryOrUrl?: string | URL | WorkerFactory, timeoutMs = 60_000) {
    this.timeoutMs = timeoutMs;
    if (typeof factoryOrUrl === 'function') {
      this.workerFactory = factoryOrUrl;
    } else {
      const url = factoryOrUrl || new URL('./worker.js', import.meta.url);
      this.workerFactory = () => new Worker(url, { type: 'module' });
    }
  }

  private ensureWorker(): Worker {
    if (this.state === 'DISPOSED') {
      throw new HeicError('WorkerConverter has been disposed', 'INTERNAL_ERROR');
    }
    if (this.worker && this.state === 'READY') {
      return this.worker;
    }

    if (this.worker) {
      try {
        this.worker.terminate();
      } catch {}
      this.worker = null;
    }

    this.worker = this.workerFactory();
    this.worker.onmessage = this.handleMessage.bind(this);
    this.worker.onerror = this.handleError.bind(this);
    this.state = 'READY';
    return this.worker;
  }

  private handleMessage(event: MessageEvent<WorkerResponse>): void {
    const res = event.data;
    if (!res || !res.id) return;

    const ctx = this.tracker.take(res.id);
    if (!ctx) return;

    if (res.success) {
      ctx.resolve((res.payload as any).result ?? res.payload);
    } else {
      ctx.reject(fromWasmError(res.error));
    }
  }

  private handleError(event: ErrorEvent): void {
    this.state = 'FAILED';
    const crashError = new HeicError(event.message || 'Worker thread crashed unexpectedly', 'INTERNAL_ERROR');
    this.tracker.rejectAll(crashError);

    if (this.worker) {
      try {
        this.worker.terminate();
      } catch {}
      this.worker = null;
    }
  }

  public send<T>(reqBuilder: (id: string) => WorkerRequest, transfer: Transferable[] = [], signal?: AbortSignal): Promise<T> {
    checkAborted(signal);
    const worker = this.ensureWorker();

    this.requestCounter += 1;
    const id = `req_${this.requestCounter}_${Date.now()}`;
    const req = reqBuilder(id);

    return new Promise<T>((resolve, reject) => {
      this.tracker.create(id, resolve, reject, this.timeoutMs, signal, () => {
        try {
          worker.postMessage({ id, type: 'abort' });
        } catch {}
      });

      worker.postMessage(req, transfer);
    });
  }

  public async detect(input: InputSource): Promise<boolean> {
    const bytes = await toUint8Array(input);
    const arrayBuf = (bytes.buffer as ArrayBuffer).slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
    return this.send((id) => ({ id, type: 'detect', data: arrayBuf }), [arrayBuf]);
  }

  public async inspect(input: InputSource, limits?: ProcessingLimits): Promise<InspectResult> {
    const bytes = await toUint8Array(input);
    const arrayBuf = (bytes.buffer as ArrayBuffer).slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
    return this.send((id) => ({ id, type: 'inspect', data: arrayBuf, limits }), [arrayBuf]);
  }

  public async convert(input: InputSource, options: ConvertOptions = {}): Promise<Blob> {
    const bytes = await toUint8Array(input);
    const arrayBuf = (bytes.buffer as ArrayBuffer).slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
    const res = await this.send<ArrayBuffer>(
      (id) => ({ id, type: 'convert', data: arrayBuf, options }),
      [arrayBuf],
      options.signal
    );
    const mime = options.format === 'png' ? 'image/png' : options.format === 'webp' ? 'image/webp' : 'image/jpeg';
    return new Blob([res], { type: mime });
  }

  public async decode(input: InputSource, options: DecodeOptions = {}): Promise<DecodedImage> {
    const bytes = await toUint8Array(input);
    const arrayBuf = (bytes.buffer as ArrayBuffer).slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
    return this.send<DecodedImage>(
      (id) => ({ id, type: 'decode', data: arrayBuf, options }),
      [arrayBuf],
      options.signal
    );
  }

  public terminate(): void {
    this.state = 'DISPOSED';
    this.tracker.rejectAll(new HeicError('Worker terminated', 'INTERNAL_ERROR'));
    if (this.worker) {
      try {
        this.worker.terminate();
      } catch {}
      this.worker = null;
    }
  }
}
