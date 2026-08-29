import test from 'node:test';
import assert from 'node:assert/strict';
import {
  HeicError,
  UnsupportedFormatError,
  LimitsExceededError,
  DecodeError,
  OperationAbortedError,
  fromWasmError,
} from '../errors.js';

test('error classes have proper names and error codes', () => {
  const err1 = new UnsupportedFormatError('Unsupported brand');
  assert.equal(err1.name, 'UnsupportedFormatError');
  assert.equal(err1.code, 'UNSUPPORTED_FORMAT');
  assert.ok(err1 instanceof HeicError);

  const err2 = new LimitsExceededError('Dimensions too large', 'LIMIT_DIMENSIONS');
  assert.equal(err2.name, 'LimitsExceededError');
  assert.equal(err2.code, 'LIMIT_DIMENSIONS');
  assert.ok(err2 instanceof HeicError);

  const err3 = new OperationAbortedError();
  assert.equal(err3.name, 'OperationAbortedError');
  assert.equal(err3.code, 'OPERATION_ABORTED');
});

test('fromWasmError maps structured WASM errors accurately', () => {
  const wasmError = new Error('Image dimensions exceed maximum');
  (wasmError as any).code = 'LIMIT_DIMENSIONS';

  const mapped = fromWasmError(wasmError);
  assert.ok(mapped instanceof LimitsExceededError);
  assert.equal(mapped.code, 'LIMIT_DIMENSIONS');

  const decodeWasmError = new Error('Corrupt slice header');
  (decodeWasmError as any).code = 'DECODE_FAILED';

  const mappedDecode = fromWasmError(decodeWasmError);
  assert.ok(mappedDecode instanceof DecodeError);
  assert.equal(mappedDecode.code, 'DECODE_FAILED');
});
