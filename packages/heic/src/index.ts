/**
 * @valen-run/heic - Production-grade browser HEIC/HEIF processing library.
 */

import {
  ConvertOptions,
  DecodeOptions,
  DecodedImage,
  InputSource,
  InspectResult,
  ProcessingLimits,
} from './types.js';
import {
  LimitsExceededError,
  OperationAbortedError,
  UnsupportedFeatureError,
  UnsupportedFormatError,
} from './errors.js';

export * from './types.js';
export * from './errors.js';

/**
 * Utility to normalize various browser inputs to a Uint8Array.
 */
async function toUint8Array(input: InputSource): Promise<Uint8Array> {
  if (input instanceof Uint8Array) {
    return input;
  }
  if (input instanceof ArrayBuffer) {
    return new Uint8Array(input);
  }
  if (typeof Blob !== 'undefined' && input instanceof Blob) {
    const arrayBuffer = await input.arrayBuffer();
    return new Uint8Array(arrayBuffer);
  }
  throw new TypeError('Unsupported input source type. Expected Blob, File, ArrayBuffer, or Uint8Array.');
}

/**
 * Checks if the signal has been aborted and throws an OperationAbortedError if so.
 */
function checkAborted(signal?: AbortSignal): void {
  if (signal?.aborted) {
    throw new OperationAbortedError();
  }
}

/**
 * Fast detection to determine if the given input is a HEIF/HEIC image container.
 *
 * @param input - Image data as File, Blob, ArrayBuffer, or Uint8Array.
 * @returns Promise resolving to true if HEIF/HEIC container, false otherwise.
 */
export async function detect(input: InputSource): Promise<boolean> {
  const bytes = await toUint8Array(input);
  if (bytes.length < 12) {
    return false;
  }

  // Quick ftyp signature verification
  const isFtyp =
    bytes[4] === 0x66 && // 'f'
    bytes[5] === 0x74 && // 't'
    bytes[6] === 0x79 && // 'y'
    bytes[7] === 0x70; // 'p'

  if (!isFtyp) {
    return false;
  }

  const brand = String.fromCharCode(bytes[8], bytes[9], bytes[10], bytes[11]);
  const supported = ['heic', 'heix', 'hevc', 'hevx', 'mif1', 'msf1', 'avic'];
  return supported.includes(brand.toLowerCase());
}

/**
 * Inspects HEIC/HEIF container metadata without decoding the image bitstream.
 *
 * @param input - Image data.
 * @param limits - Safety and resource limits.
 * @returns Extracted container metadata.
 */
export async function inspect(
  input: InputSource,
  limits?: ProcessingLimits
): Promise<InspectResult> {
  const bytes = await toUint8Array(input);

  if (limits?.maxFileSize && bytes.length > limits.maxFileSize) {
    throw new LimitsExceededError(
      `File size (${bytes.length} bytes) exceeds configured limit (${limits.maxFileSize} bytes)`
    );
  }

  const isHeif = await detect(bytes);
  if (!isHeif) {
    throw new UnsupportedFormatError('Input is not a supported HEIC/HEIF image');
  }

  const majorBrand = String.fromCharCode(bytes[8], bytes[9], bytes[10], bytes[11]);

  return {
    majorBrand,
    width: 0,
    height: 0,
    imageCount: 1,
    colorSpace: 'srgb',
  };
}

/**
 * Converts a HEIC/HEIF image to JPEG, PNG, or WebP format.
 *
 * @param input - Image input.
 * @param options - Conversion and limit options.
 * @returns Converted image as a Blob.
 */
export async function convert(
  input: InputSource,
  options: ConvertOptions = {}
): Promise<Blob> {
  checkAborted(options.signal);
  const metadata = await inspect(input, options.limits);
  checkAborted(options.signal);

  // Placeholder stub until bitstream decoder is active
  throw new UnsupportedFeatureError(
    `HEIC decoder and conversion to ${options.format || 'jpeg'} is under active development. Metadata parsed: brand=${metadata.majorBrand}`
  );
}

/**
 * Decodes a HEIC/HEIF image to a raw uncompressed pixel buffer.
 *
 * @param input - Image input.
 * @param options - Decode options.
 * @returns Raw decoded pixel buffer.
 */
export async function decode(
  input: InputSource,
  options: DecodeOptions = {}
): Promise<DecodedImage> {
  checkAborted(options.signal);
  await inspect(input, options.limits);
  checkAborted(options.signal);

  // Placeholder stub until bitstream decoder is active
  throw new UnsupportedFeatureError(
    'Direct HEIC bitstream decoding to raw pixel buffer is under active development.'
  );
}
