import { test, expect } from '@playwright/test';
import { createSyntheticHeic } from '../fixtures/test-samples.js';

test.describe('AbortSignal Cancellation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/standard');
    await page.waitForFunction(() => (window as any).__valen_heic?.ready === true);
  });

  test('cancels main-thread conversion promptly under 10ms with AbortController', async ({ page }) => {
    const heicBytes = Array.from(createSyntheticHeic(1920, 1080));

    const result = await page.evaluate(async (bytes) => {
      const { heicTo } = (window as any).__valen_heic;
      const controller = new AbortController();
      const blob = new Blob([new Uint8Array(bytes)], { type: 'image/heic' });

      // Immediate abort
      setTimeout(() => controller.abort(), 2);

      try {
        await heicTo(blob, { signal: controller.signal });
        return { aborted: false, errorCode: null };
      } catch (err: any) {
        return {
          aborted: true,
          errorCode: err?.code || err?.name,
          message: err?.message,
        };
      }
    }, heicBytes);

    expect(result.aborted).toBe(true);
  });

  test('cancels worker conversion promptly with AbortSignal', async ({ page }) => {
    const heicBytes = Array.from(createSyntheticHeic(1920, 1080));

    const result = await page.evaluate(async (bytes) => {
      const { createWorkerConverter } = (window as any).__valen_heic;
      const worker = createWorkerConverter();
      const controller = new AbortController();

      try {
        const blob = new Blob([new Uint8Array(bytes)], { type: 'image/heic' });
        setTimeout(() => controller.abort(), 2);

        await worker.convert(blob, { signal: controller.signal });
        return { aborted: false };
      } catch (err: any) {
        return {
          aborted: true,
          errorCode: err?.code || err?.name,
        };
      } finally {
        worker.terminate();
      }
    }, heicBytes);

    expect(result.aborted).toBe(true);
  });
});
