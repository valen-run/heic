/**
 * Unified error hierarchy and WASM error translation for `@valen-run/heic`.
 */

export type HeicErrorCode =
  | 'INVALID_INPUT'
  | 'INVALID_CONTAINER'
  | 'MALFORMED_INPUT'
  | 'UNSUPPORTED_FORMAT'
  | 'UNSUPPORTED_BRAND'
  | 'UNSUPPORTED_CODEC'
  | 'UNSUPPORTED_FEATURE'
  | 'LIMIT_INPUT_BYTES'
  | 'LIMIT_DIMENSIONS'
  | 'LIMIT_PIXELS'
  | 'LIMIT_MEMORY'
  | 'LIMIT_EXCEEDED'
  | 'DECODE_FAILED'
  | 'ENCODE_FAILED'
  | 'OPERATION_ABORTED'
  | 'INVALID_OPTIONS'
  | 'INTERNAL_ERROR';

/**
 * Base error class for all HEIC processing errors.
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

export class UnsupportedFormatError extends HeicError {
  constructor(message = 'The provided file format or HEIF brand is not supported', code: HeicErrorCode = 'UNSUPPORTED_FORMAT') {
    super(message, code);
    this.name = 'UnsupportedFormatError';
  }
}

export class InvalidContainerError extends HeicError {
  constructor(message = 'Invalid or corrupt HEIF/HEIC container structure', code: HeicErrorCode = 'INVALID_CONTAINER') {
    super(message, code);
    this.name = 'InvalidContainerError';
  }
}

export class MalformedInputError extends HeicError {
  constructor(message = 'Malformed input data encountered', code: HeicErrorCode = 'MALFORMED_INPUT') {
    super(message, code);
    this.name = 'MalformedInputError';
  }
}

export class LimitsExceededError extends HeicError {
  constructor(message = 'Configured resource limits exceeded', code: HeicErrorCode = 'LIMIT_EXCEEDED') {
    super(message, code);
    this.name = 'LimitsExceededError';
  }
}

export class DecodeError extends HeicError {
  constructor(message = 'Failed to decode HEIC/HEVC bitstream', code: HeicErrorCode = 'DECODE_FAILED') {
    super(message, code);
    this.name = 'DecodeError';
  }
}

export class EncodeError extends HeicError {
  constructor(message = 'Failed to encode image to target format', code: HeicErrorCode = 'ENCODE_FAILED') {
    super(message, code);
    this.name = 'EncodeError';
  }
}

export class OperationAbortedError extends HeicError {
  constructor(message = 'Operation was aborted') {
    super(message, 'OPERATION_ABORTED');
    this.name = 'OperationAbortedError';
  }
}

export class InvalidOptionsError extends HeicError {
  constructor(message = 'Invalid options provided') {
    super(message, 'INVALID_OPTIONS');
    this.name = 'InvalidOptionsError';
  }
}

/**
 * Translates errors thrown across the WASM boundary into typed `HeicError` instances.
 */
export function fromWasmError(err: unknown): HeicError {
  if (err instanceof HeicError) {
    return err;
  }

  const rawMsg = err instanceof Error ? err.message : String(err);
  const code = (err && typeof err === 'object' && 'code' in err ? String((err as any).code) : '') as HeicErrorCode;

  switch (code) {
    case 'INVALID_INPUT':
      return new HeicError(rawMsg, 'INVALID_INPUT');
    case 'INVALID_CONTAINER':
      return new InvalidContainerError(rawMsg, 'INVALID_CONTAINER');
    case 'MALFORMED_INPUT':
      return new MalformedInputError(rawMsg, 'MALFORMED_INPUT');
    case 'UNSUPPORTED_FORMAT':
    case 'UNSUPPORTED_BRAND':
    case 'UNSUPPORTED_CODEC':
      return new UnsupportedFormatError(rawMsg, code);
    case 'LIMIT_INPUT_BYTES':
    case 'LIMIT_DIMENSIONS':
    case 'LIMIT_PIXELS':
    case 'LIMIT_MEMORY':
    case 'LIMIT_EXCEEDED':
      return new LimitsExceededError(rawMsg, code);
    case 'DECODE_FAILED':
      return new DecodeError(rawMsg, 'DECODE_FAILED');
    case 'ENCODE_FAILED':
      return new EncodeError(rawMsg, 'ENCODE_FAILED');
    case 'INVALID_OPTIONS':
      return new InvalidOptionsError(rawMsg);
    case 'OPERATION_ABORTED':
      return new OperationAbortedError(rawMsg);
    default:
      return new HeicError(rawMsg, 'INTERNAL_ERROR');
  }
}
