/**
 * Result models, metadata descriptors, and pixel formats.
 */

/**
 * Standard image output formats.
 */
export type OutputFormat = 'jpeg' | 'png' | 'webp';

/**
 * Pixel format of decoded pixel buffers.
 */
export type PixelFormat = 'rgb8' | 'rgba8' | 'bgr8' | 'bgra8';

/**
 * Metadata extracted from HEIF container inspection.
 */
export interface InspectResult {
  /** Major brand identifier (e.g. 'heic', 'mif1') */
  majorBrand: string;
  /** Compatible brand array */
  compatibleBrands?: string[];
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
  /** Total number of image items */
  imageCount: number;
  /** EXIF orientation value (1-8) if present */
  orientation?: number;
  /** Color profile identifier if detected */
  colorSpace?: 'srgb' | 'display-p3' | 'rec2020' | 'icc' | string;
  /** Whether the primary image has an auxiliary alpha channel */
  hasAlpha?: boolean;
  /** Whether this is an image grid */
  isGrid?: boolean;
  /** Number of grid rows if grid */
  gridRows?: number;
  /** Number of grid columns if grid */
  gridColumns?: number;
}

/**
 * Decoded uncompressed image pixel buffer.
 */
export interface DecodedImage {
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
  /** Pixel format (e.g. 'rgba8') */
  format: PixelFormat;
  /** Row stride in bytes */
  stride: number;
  /** Interleaved raw pixel byte array */
  data: Uint8Array;
}
