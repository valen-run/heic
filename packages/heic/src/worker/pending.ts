/**
 * In-flight request tracking, timeout management, and signal cleanup for worker client.
 */

import { HeicError, OperationAbortedError } from '../errors.js';

export interface PendingRequest {
  resolve: (val: any) => void;
  reject: (err: any) => void;
  timeoutId?: any;
  signalListener?: () => void;
  signal?: AbortSignal;
}

/**
 * Manages active in-flight worker requests with cancellation and timeout handlers.
 */
export class PendingRequestTracker {
  private pending = new Map<string, PendingRequest>();

  public create(
    id: string,
    resolve: (val: any) => void,
    reject: (err: any) => void,
    timeoutMs: number,
    signal?: AbortSignal,
    onAbort?: () => void
  ): void {
    const ctx: PendingRequest = { resolve, reject, signal };

    if (timeoutMs > 0) {
      ctx.timeoutId = setTimeout(() => {
        this.cleanup(id);
        reject(new HeicError(`Worker request ${id} timed out after ${timeoutMs}ms`, 'INTERNAL_ERROR'));
      }, timeoutMs);
    }

    if (signal) {
      ctx.signalListener = () => {
        this.cleanup(id);
        if (onAbort) onAbort();
        reject(new OperationAbortedError());
      };
      signal.addEventListener('abort', ctx.signalListener, { once: true });
    }

    this.pending.set(id, ctx);
  }

  public take(id: string): PendingRequest | undefined {
    const ctx = this.pending.get(id);
    if (ctx) {
      this.cleanup(id);
    }
    return ctx;
  }

  public cleanup(id: string): void {
    const ctx = this.pending.get(id);
    if (!ctx) return;
    this.pending.delete(id);
    if (ctx.timeoutId) clearTimeout(ctx.timeoutId);
    if (ctx.signal && ctx.signalListener) {
      ctx.signal.removeEventListener('abort', ctx.signalListener);
    }
  }

  public rejectAll(error: Error): void {
    for (const [id, ctx] of this.pending.entries()) {
      this.cleanup(id);
      ctx.reject(error);
    }
  }

  public get size(): number {
    return this.pending.size;
  }
}
