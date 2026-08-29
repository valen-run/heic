/**
 * Conversion, decode, and safety limit option interfaces.
 */

import type { OutputFormat, PixelFormat } from './results.js';

/**
 * Valid input representations supported by the library.
 */
export type InputSource = Blob | File | ArrayBuffer | Uint8Array;

/**
 * Configurable safety and resource limits to guard against resource exhaustion.
 */
export interface ProcessingLimits {
  /** Maximum allowed input file size in bytes */
  maxFileSize?: number;
  /** Maximum allowed image width in pixels */
  maxWidth?: number;
  /** Maximum allowed image height in pixels */
  maxHeight?: number;
  /** Maximum allowed total pixel count (width * height) */
  maxPixelCount?: number;
  /** Maximum memory allocation in bytes for decoded buffers */
  maxMemoryBytes?: number;
}

/**
 * General conversion options for HEIC to standard image formats.
 */
export interface ConvertOptions {
  /** Target image format (default: 'jpeg') */
  format?: OutputFormat;
  /** Compression quality between 0.0 and 1.0 (or 1 to 100) */
  quality?: number;
  /** Solid RGB background color [r, g, b] used when flattening alpha for JPEG */
  backgroundColor?: [number, number, number];
  /** Whether to rotate the image according to EXIF orientation (default: true) */
  applyOrientation?: boolean;
  /** Resource and safety limits */
  limits?: ProcessingLimits;
  /** Optional cancellation signal */
  signal?: AbortSignal;
}

/**
 * Options for the main `heicTo` interface.
 */
export interface HeicToOptions extends ConvertOptions {
  /** Output format type identifier or MIME string */
  type?: 'blob' | 'image/jpeg' | 'image/png' | 'image/webp' | 'png' | 'jpeg' | 'jpg' | 'webp' | 'bitmap' | 'raw';
}

/**
 * Options for direct bitstream decoding to uncompressed raw pixels.
 */
export interface DecodeOptions {
  /** Target pixel format (default: 'rgba8') */
  pixelFormat?: PixelFormat;
  /** Whether to rotate/flip the decoded buffer according to EXIF orientation (default: true) */
  applyOrientation?: boolean;
  /** Resource limits to enforce */
  limits?: ProcessingLimits;
  /** Optional cancellation signal */
  signal?: AbortSignal;
}
