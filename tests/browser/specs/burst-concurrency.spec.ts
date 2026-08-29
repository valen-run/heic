import { test, expect } from '@playwright/test';
import { createSyntheticHeic } from '../fixtures/test-samples.js';

test.describe('Burst Concurrency', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/standard');
    await page.waitForFunction(() => (window as any).__valen_heic?.ready === true);
  });

  test('executes 50 parallel conversions without deadlocks or crashes', async ({ page }) => {
    const heicBytes = Array.from(createSyntheticHeic(320, 240));

    const result = await page.evaluate(async (bytes) => {
      const { heicTo } = (window as any).__valen_heic;
      const count = 50;
      const promises: Promise<Blob>[] = [];

      for (let i = 0; i < count; i++) {
        const blob = new Blob([new Uint8Array(bytes)], { type: 'image/heic' });
        promises.push(heicTo(blob, { type: 'image/jpeg' }) as Promise<Blob>);
      }

      const results = await Promise.all(promises);
      return {
        completed: results.length,
        allBlobs: results.every((r) => r instanceof Blob && r.size > 0),
      };
    }, heicBytes);

    expect(result.completed).toBe(50);
    expect(result.allBlobs).toBe(true);
  });
});
