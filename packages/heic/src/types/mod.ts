/**
 * Type declarations and discriminated type overloads for `@valen-run/heic`.
 */

export * from './options.js';
export * from './results.js';

import type { HeicToOptions, InputSource } from './options.js';
import type { DecodedImage } from './results.js';

/**
 * Discriminated overload signatures for `heicTo`.
 */
export type HeicToReturnType<T extends HeicToOptions> = T extends { type: 'bitmap' }
  ? ImageBitmap
  : T extends { type: 'raw' }
  ? DecodedImage
  : Blob;

/**
 * Main `heicTo` function signature.
 */
export type HeicToFn = {
  <T extends HeicToOptions>(input: InputSource, options?: T): Promise<HeicToReturnType<T>>;
};
