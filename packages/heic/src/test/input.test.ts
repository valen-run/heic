import test from 'node:test';
import assert from 'node:assert/strict';
import { toUint8Array, toUint8ArraySync, checkAborted } from '../input.js';
import { OperationAbortedError } from '../errors.js';

test('input normalization accepts Uint8Array', async () => {
  const input = new Uint8Array([1, 2, 3, 4]);
  const result = await toUint8Array(input);
  assert.equal(result.length, 4);
  assert.equal(result[0], 1);
  assert.equal(result[3], 4);
});

test('input normalization accepts ArrayBuffer', async () => {
  const buf = new ArrayBuffer(8);
  const view = new Uint8Array(buf);
  view.fill(42);
  const result = await toUint8Array(buf);
  assert.equal(result.length, 8);
  assert.equal(result[0], 42);
});

test('input normalization rejects invalid types with TypeError', async () => {
  await assert.rejects(async () => {
    // @ts-expect-error testing invalid input type
    await toUint8Array('not a buffer');
  }, TypeError);

  await assert.rejects(async () => {
    // @ts-expect-error testing invalid input type
    await toUint8Array(12345);
  }, TypeError);

  await assert.rejects(async () => {
    // @ts-expect-error testing invalid input type
    await toUint8Array(null);
  }, TypeError);
});

test('synchronous normalization operates on binary buffers', () => {
  const input = new Uint8Array([10, 20]);
  const res = toUint8ArraySync(input);
  assert.equal(res.length, 2);
  assert.equal(res[0], 10);

  assert.throws(() => {
    // @ts-expect-error testing invalid sync input
    toUint8ArraySync('invalid');
  }, TypeError);
});

test('checkAborted throws OperationAbortedError when signal is aborted', () => {
  const controller = new AbortController();
  assert.doesNotThrow(() => checkAborted(controller.signal));

  controller.abort();
  assert.throws(() => checkAborted(controller.signal), OperationAbortedError);
});
