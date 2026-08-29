import { test, expect } from '@playwright/test';
import { createSyntheticHeic } from '../fixtures/test-samples.js';

test.describe('Worker-Thread Conversion', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/standard');
    await page.waitForFunction(() => (window as any).__valen_heic?.ready === true);
  });

  test('converts HEIC to JPEG off-thread using WorkerConverter', async ({ page }) => {
    const heicBytes = Array.from(createSyntheticHeic(800, 600));

    const result = await page.evaluate(async (bytes) => {
      const { createWorkerConverter } = (window as any).__valen_heic;
      const worker = createWorkerConverter();
      try {
        const blob = new Blob([new Uint8Array(bytes)], { type: 'image/heic' });
        const converted = await worker.convert(blob, { format: 'jpeg', quality: 85 });
        const buf = new Uint8Array(await converted.arrayBuffer());

        return {
          type: converted.type,
          size: converted.size,
          isJpeg: buf[0] === 0xff && buf[1] === 0xd8,
        };
      } finally {
        worker.terminate();
      }
    }, heicBytes);

    expect(result.type).toBe('image/jpeg');
    expect(result.size).toBeGreaterThan(0);
    expect(result.isJpeg).toBe(true);
  });

  test('decodes raw pixels in Web Worker', async ({ page }) => {
    const heicBytes = Array.from(createSyntheticHeic(320, 240));

    const result = await page.evaluate(async (bytes) => {
      const { createWorkerConverter } = (window as any).__valen_heic;
      const worker = createWorkerConverter();
      try {
        const blob = new Blob([new Uint8Array(bytes)], { type: 'image/heic' });
        const decoded = await worker.decode(blob);

        return {
          width: decoded.width,
          height: decoded.height,
          dataLength: decoded.data.length,
          format: decoded.format,
        };
      } finally {
        worker.terminate();
      }
    }, heicBytes);

    expect(result.width).toBe(320);
    expect(result.height).toBe(240);
    expect(result.dataLength).toBe(320 * 240 * 4);
  });
});
