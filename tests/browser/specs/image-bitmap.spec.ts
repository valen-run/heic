import { test, expect } from '@playwright/test';
import { createSyntheticHeic } from '../fixtures/test-samples.js';

test.describe('ImageBitmap Conversion', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/standard');
    await page.waitForFunction(() => (window as any).__valen_heic?.ready === true);
  });

  test('converts HEIC directly to ImageBitmap and renders to Canvas', async ({ page }) => {
    const heicBytes = Array.from(createSyntheticHeic(640, 480));

    const result = await page.evaluate(async (bytes) => {
      const { heicTo } = (window as any).__valen_heic;
      const blob = new Blob([new Uint8Array(bytes)], { type: 'image/heic' });
      const bitmap = await heicTo(blob, { type: 'bitmap' });

      const isBitmap = bitmap instanceof ImageBitmap;
      const width = bitmap.width;
      const height = bitmap.height;

      // Draw onto DOM canvas
      const canvas = document.getElementById('test-canvas') as HTMLCanvasElement;
      const ctx = canvas.getContext('2d')!;
      ctx.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
      bitmap.close();

      return {
        isBitmap,
        width,
        height,
      };
    }, heicBytes);

    expect(result.isBitmap).toBe(true);
    expect(result.width).toBe(640);
    expect(result.height).toBe(480);
  });
});
