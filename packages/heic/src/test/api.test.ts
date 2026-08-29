import test from 'node:test';
import assert from 'node:assert/strict';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  isHeic,
  isHeicSync,
  heicTo,
  probe,
  setWasmSource,
  ensureInitialized,
  OperationAbortedError,
  HeicError,
} from '../index.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const wasmPath = path.resolve(__dirname, '../../pkg/valen_heic_wasm_bg.wasm');

test('WASM initialization and container detection', async () => {
  const wasmBuffer = fs.readFileSync(wasmPath);
  setWasmSource(new Uint8Array(wasmBuffer));
  await ensureInitialized();

  // Valid HEIC ftyp box: 16 bytes: length=16, type='ftyp', major_brand='heic'
  const validHeic = new Uint8Array([
    0x00, 0x00, 0x00, 0x10,
    0x66, 0x74, 0x79, 0x70, // 'ftyp'
    0x68, 0x65, 0x69, 0x63, // 'heic'
    0x00, 0x00, 0x00, 0x00,
  ]);

  assert.equal(isHeicSync(validHeic), true);
  assert.equal(await isHeic(validHeic), true);

  const invalid = new Uint8Array([0x00, 0x00, 0x00, 0x10, 0x72, 0x61, 0x6e, 0x64]);
  assert.equal(isHeicSync(invalid), false);
  assert.equal(await isHeic(invalid), false);
});

test('heicTo rejects non-binary inputs with TypeError', async () => {
  await assert.rejects(async () => {
    // @ts-expect-error testing invalid argument
    await heicTo('not-a-file');
  }, TypeError);
});

test('heicTo respects AbortSignal cancellation', async () => {
  const controller = new AbortController();
  controller.abort();

  const dummy = new Uint8Array([0, 0, 0, 16, 0x66, 0x74, 0x79, 0x70, 0x68, 0x65, 0x69, 0x63, 0, 0, 0, 0]);

  await assert.rejects(async () => {
    await heicTo(dummy, { signal: controller.signal });
  }, OperationAbortedError);
});

test('probe rejects corrupted HEIC with HeicError', async () => {
  const truncated = new Uint8Array([
    0x00, 0x00, 0x00, 0x10,
    0x66, 0x74, 0x79, 0x70,
    0x68, 0x65, 0x69, 0x63,
    0x00, 0x00, 0x00, 0x00,
  ]);

  await assert.rejects(async () => {
    await probe(truncated);
  }, HeicError);
});
