/**
 * Unified error hierarchy for `@valen-run/heic`.
 */

export type HeicErrorCode =
  | 'UNSUPPORTED_FORMAT'
  | 'INVALID_CONTAINER'
  | 'MALFORMED_INPUT'
  | 'LIMIT_EXCEEDED'
  | 'PIXEL_LIMIT_EXCEEDED'
  | 'DECODE_ERROR'
  | 'ENCODE_ERROR'
  | 'OPERATION_ABORTED'
  | 'UNSUPPORTED_FEATURE'
  | 'INTERNAL_ERROR';

/**
 * Base error class for all HEIC processing failures.
 */
export class HeicError extends Error {
  readonly code: HeicErrorCode;

  constructor(message: string, code: HeicErrorCode = 'INTERNAL_ERROR') {
    super(message);
    this.name = 'HeicError';
    this.code = code;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

/**
 * Thrown when the provided input format or brand is unsupported.
 */
export class UnsupportedFormatError extends HeicError {
  constructor(message = 'The provided file format or HEIF brand is not supported') {
    super(message, 'UNSUPPORTED_FORMAT');
    this.name = 'UnsupportedFormatError';
  }
}

/**
 * Thrown when container structures (e.g. ISOBMFF boxes) are corrupt or missing.
 */
export class InvalidContainerError extends HeicError {
  constructor(message = 'Invalid or corrupt HEIF/HEIC container structure') {
    super(message, 'INVALID_CONTAINER');
    this.name = 'InvalidContainerError';
  }
}

/**
 * Thrown when input data is truncated or malformed.
 */
export class MalformedInputError extends HeicError {
  constructor(message = 'Malformed input data encountered') {
    super(message, 'MALFORMED_INPUT');
    this.name = 'MalformedInputError';
  }
}

/**
 * Thrown when resource limits (file size, dimensions, memory) are exceeded.
 */
export class LimitsExceededError extends HeicError {
  constructor(message = 'Configured resource limits exceeded') {
    super(message, 'LIMIT_EXCEEDED');
    this.name = 'LimitsExceededError';
  }
}

/**
 * Thrown when maximum allowed pixel count is exceeded.
 */
export class PixelLimitExceededError extends LimitsExceededError {
  readonly pixelCount?: number;
  readonly maxPixelCount?: number;

  constructor(pixelCount?: number, maxPixelCount?: number) {
    const msg =
      pixelCount && maxPixelCount
        ? `Pixel count (${pixelCount}) exceeds maximum allowed (${maxPixelCount})`
        : 'Pixel limit exceeded';
    super(msg);
    this.name = 'PixelLimitExceededError';
    this.pixelCount = pixelCount;
    this.maxPixelCount = maxPixelCount;
  }
}

/**
 * Thrown when decoding the compressed bitstream fails.
 */
export class DecodeError extends HeicError {
  constructor(message = 'Failed to decode HEIC/HEVC bitstream') {
    super(message, 'DECODE_ERROR');
    this.name = 'DecodeError';
  }
}

/**
 * Thrown when encoding raw pixel buffers to target format fails.
 */
export class EncodeError extends HeicError {
  constructor(message = 'Failed to encode image to target format') {
    super(message, 'ENCODE_ERROR');
    this.name = 'EncodeError';
  }
}

/**
 * Thrown when an asynchronous operation is cancelled via `AbortSignal`.
 */
export class OperationAbortedError extends HeicError {
  constructor(message = 'Operation was aborted') {
    super(message, 'OPERATION_ABORTED');
    this.name = 'OperationAbortedError';
  }
}

/**
 * Thrown when an image feature is valid in specification but not supported by this library.
 */
export class UnsupportedFeatureError extends HeicError {
  constructor(message = 'Unsupported HEIC/HEIF feature') {
    super(message, 'UNSUPPORTED_FEATURE');
    this.name = 'UnsupportedFeatureError';
  }
}
