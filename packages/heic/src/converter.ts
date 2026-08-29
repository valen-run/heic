/**
 * High-level conversion orchestrator and format synthesis.
 */

import { wasmConvert, wasmGetRawPixels } from './bridge.js';
import { checkAborted, toUint8Array } from './input.js';
import type { HeicToOptions, InputSource } from './types/options.js';
import type { DecodedImage } from './types/results.js';

function resolveMimeType(type?: string): { mime: string; format: 'jpeg' | 'png' | 'webp' } {
  switch (type?.toLowerCase()) {
    case 'png':
    case 'image/png':
      return { mime: 'image/png', format: 'png' };
    case 'webp':
    case 'image/webp':
      return { mime: 'image/webp', format: 'webp' };
    case 'jpeg':
    case 'jpg':
    case 'image/jpeg':
    case 'image/jpg':
    case 'blob':
    default:
      return { mime: 'image/jpeg', format: 'jpeg' };
  }
}

/**
 * Main conversion function converting HEIC/HEIF images into Blob, ImageBitmap, or DecodedImage.
 */
export async function heicTo(
  input: InputSource,
  options: HeicToOptions = {}
): Promise<Blob | ImageBitmap | DecodedImage> {
  checkAborted(options.signal);
  const bytes = await toUint8Array(input);
  checkAborted(options.signal);

  // 1. Raw uncompressed pixel buffer output
  if (options.type === 'raw') {
    const rawImage = await wasmGetRawPixels(bytes, {
      pixelFormat: 'rgba8',
      applyOrientation: options.applyOrientation ?? true,
      limits: options.limits,
      signal: options.signal,
    });
    checkAborted(options.signal);
    return rawImage;
  }

  // 2. High-performance Browser ImageBitmap output directly from raw RGBA
  if (options.type === 'bitmap') {
    const raw = await wasmGetRawPixels(bytes, {
      pixelFormat: 'rgba8',
      applyOrientation: options.applyOrientation ?? true,
      limits: options.limits,
      signal: options.signal,
    });
    checkAborted(options.signal);

    if (typeof createImageBitmap !== 'undefined' && typeof ImageData !== 'undefined') {
      const clamped = new Uint8ClampedArray(
        raw.data.buffer as ArrayBuffer,
        raw.data.byteOffset,
        raw.data.byteLength
      );
      const imgData = new ImageData(clamped, raw.width, raw.height);
      return await createImageBitmap(imgData);
    }

    return raw as any;
  }

  // 3. Standard Blob output (JPEG, PNG, WebP)
  const { mime, format } = resolveMimeType(options.type || options.format);
  const encodedBytes = await wasmConvert(bytes, {
    ...options,
    format,
  });
  checkAborted(options.signal);

  if (typeof Blob !== 'undefined') {
    return new Blob([encodedBytes as Uint8Array<ArrayBuffer>], { type: mime });
  }

  // Fallback for environments without DOM Blob
  return encodedBytes as any;
}
