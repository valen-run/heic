/**
 * Core type declarations for `@valen-run/heic`.
 */

/**
 * Valid input representations supported by the library.
 */
export type InputSource = Blob | File | ArrayBuffer | Uint8Array;

/**
 * Target output format for conversion.
 */
export type OutputFormat = 'jpeg' | 'png' | 'webp';

/**
 * Pixel format of decoded pixel buffer.
 */
export type PixelFormat = 'rgb8' | 'rgba8';

/**
 * Configurable resource limits to guard against resource exhaustion.
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
  /** Maximum memory allocation in bytes for internal decoded buffers */
  maxMemoryBytes?: number;
}

/**
 * Metadata extracted from HEIF container inspection.
 */
export interface InspectResult {
  /** Primary major brand identifier (e.g. 'heic', 'mif1') */
  majorBrand: string;
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
  /** Total number of images contained in the file */
  imageCount: number;
  /** EXIF orientation value (1-8) if present */
  orientation?: number;
  /** Color profile identifier if detected */
  colorSpace?: 'srgb' | 'display-p3' | 'rec2020' | 'icc';
}

/**
 * Options for converting HEIC/HEIF images.
 */
export interface ConvertOptions {
  /** Target image format (default: 'jpeg') */
  format?: OutputFormat;
  /** Compression quality between 0.0 and 1.0 (for JPEG and WebP) */
  quality?: number;
  /** Resource limits to enforce */
  limits?: ProcessingLimits;
  /** Optional cancellation signal */
  signal?: AbortSignal;
}

/**
 * Options for direct bitstream decoding to raw pixels.
 */
export interface DecodeOptions {
  /** Desired pixel format */
  pixelFormat?: PixelFormat;
  /** Whether to rotate/flip the decoded buffer according to EXIF orientation */
  applyOrientation?: boolean;
  /** Resource limits to enforce */
  limits?: ProcessingLimits;
  /** Optional cancellation signal */
  signal?: AbortSignal;
}

/**
 * Decoded uncompressed image buffer.
 */
export interface DecodedImage {
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
  /** Pixel format */
  pixelFormat: PixelFormat;
  /** Interleaved raw pixel byte array */
  data: Uint8Array;
}
