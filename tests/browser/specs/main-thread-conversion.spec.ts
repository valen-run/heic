import { test, expect } from '@playwright/test';
import { createSyntheticHeic } from '../fixtures/test-samples.js';

test.describe('Main-Thread Conversion', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/standard');
    await page.waitForFunction(() => (window as any).__valen_heic?.ready === true);
  });

  test('converts HEIC to JPEG Blob on main thread', async ({ page }) => {
    const heicBytes = Array.from(createSyntheticHeic(800, 600));

    const result = await page.evaluate(async (bytes) => {
      const { heicTo } = (window as any).__valen_heic;
      const blob = new Blob([new Uint8Array(bytes)], { type: 'image/heic' });
      const converted = await heicTo(blob, { type: 'image/jpeg', quality: 0.85 });

      const buf = new Uint8Array(await converted.arrayBuffer());
      return {
        type: converted.type,
        size: converted.size,
        isJpegHeader: buf[0] === 0xff && buf[1] === 0xd8,
      };
    }, heicBytes);

    expect(result.type).toBe('image/jpeg');
    expect(result.size).toBeGreaterThan(0);
    expect(result.isJpegHeader).toBe(true);
  });

  test('converts HEIC to PNG Blob on main thread', async ({ page }) => {
    const heicBytes = Array.from(createSyntheticHeic(400, 300));

    const result = await page.evaluate(async (bytes) => {
      const { heicTo } = (window as any).__valen_heic;
      const blob = new Blob([new Uint8Array(bytes)], { type: 'image/heic' });
      const converted = await heicTo(blob, { type: 'image/png' });

      const buf = new Uint8Array(await converted.arrayBuffer());
      return {
        type: converted.type,
        size: converted.size,
        isPngHeader: buf[0] === 0x89 && buf[1] === 0x50 && buf[2] === 0x4e && buf[3] === 0x47,
      };
    }, heicBytes);

    expect(result.type).toBe('image/png');
    expect(result.size).toBeGreaterThan(0);
    expect(result.isPngHeader).toBe(true);
  });

  test('converts HEIC to WebP Blob on main thread', async ({ page }) => {
    const heicBytes = Array.from(createSyntheticHeic(400, 300));

    const result = await page.evaluate(async (bytes) => {
      const { heicTo } = (window as any).__valen_heic;
      const blob = new Blob([new Uint8Array(bytes)], { type: 'image/webp' });
      const converted = await heicTo(blob, { type: 'image/webp' });

      const buf = new Uint8Array(await converted.arrayBuffer());
      const riffHeader = String.fromCharCode(...buf.slice(0, 4));
      return {
        type: converted.type,
        size: converted.size,
        isRiff: riffHeader === 'RIFF',
      };
    }, heicBytes);

    expect(result.type).toBe('image/webp');
    expect(result.size).toBeGreaterThan(0);
    expect(result.isRiff).toBe(true);
  });
});
