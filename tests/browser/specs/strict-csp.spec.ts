import { test, expect } from '@playwright/test';
import { createSyntheticHeic } from '../fixtures/test-samples.js';

test.describe('Strict Content Security Policy (No unsafe-eval)', () => {
  test('initializes WebAssembly and converts images under strict CSP', async ({ page }) => {
    const cspErrors: string[] = [];
    page.on('console', (msg) => {
      if (msg.type() === 'error' && msg.text().includes('Content Security Policy')) {
        cspErrors.push(msg.text());
      }
    });

    await page.goto('/strict-csp');
    await page.waitForFunction(() => (window as any).__valen_heic?.ready === true);

    const heicBytes = Array.from(createSyntheticHeic(640, 480));

    const result = await page.evaluate(async (bytes) => {
      const { heicTo, isHeic } = (window as any).__valen_heic;
      const blob = new Blob([new Uint8Array(bytes)], { type: 'image/heic' });

      const detected = await isHeic(blob);
      const converted = await heicTo(blob, { type: 'image/jpeg' });

      return {
        detected,
        convertedSize: converted.size,
        convertedType: converted.type,
      };
    }, heicBytes);

    expect(cspErrors).toEqual([]);
    expect(result.detected).toBe(true);
    expect(result.convertedSize).toBeGreaterThan(0);
    expect(result.convertedType).toBe('image/jpeg');
  });

  test('runs Web Worker under strict CSP without eval or external scripts', async ({ page }) => {
    await page.goto('/strict-csp');
    await page.waitForFunction(() => (window as any).__valen_heic?.ready === true);

    const heicBytes = Array.from(createSyntheticHeic(400, 300));

    const result = await page.evaluate(async (bytes) => {
      const { createWorkerConverter } = (window as any).__valen_heic;
      const worker = createWorkerConverter();
      try {
        const blob = new Blob([new Uint8Array(bytes)], { type: 'image/heic' });
        const converted = await worker.convert(blob, { format: 'png' });
        return {
          type: converted.type,
          size: converted.size,
        };
      } finally {
        worker.terminate();
      }
    }, heicBytes);

    expect(result.type).toBe('image/png');
    expect(result.size).toBeGreaterThan(0);
  });
});
