/**
 * Browser test fixture utilities and format validation helpers.
 */

export * from './test-samples.js';

export function isJpegBlob(blob: Blob): Promise<boolean> {
  return blob.arrayBuffer().then((buf) => {
    const bytes = new Uint8Array(buf);
    return bytes[0] === 0xff && bytes[1] === 0xd8 && bytes[bytes.length - 2] === 0xff && bytes[bytes.length - 1] === 0xd9;
  });
}

export function isPngBlob(blob: Blob): Promise<boolean> {
  return blob.arrayBuffer().then((buf) => {
    const bytes = new Uint8Array(buf);
    return bytes[0] === 0x89 && bytes[1] === 0x50 && bytes[2] === 0x4e && bytes[3] === 0x47;
  });
}

export function isWebpBlob(blob: Blob): Promise<boolean> {
  return blob.arrayBuffer().then((buf) => {
    const bytes = new Uint8Array(buf);
    const text = new TextDecoder().decode(bytes.slice(0, 12));
    return text.startsWith('RIFF') && text.endsWith('WEBP');
  });
}
