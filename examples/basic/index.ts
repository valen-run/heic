/**
 * Basic Node/TypeScript usage example for @valen-run/heic.
 */

import * as fs from 'node:fs/promises';
import { detect, inspect, LimitsExceededError } from '@valen-run/heic';

async function main() {
  console.log('--- @valen-run/heic Basic Example ---');

  // Synthetic sample header
  const sampleData = new Uint8Array([
    0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, // ftyp
    0x68, 0x65, 0x69, 0x63, // heic
    0x00, 0x00, 0x00, 0x00, // minor_version
    0x6d, 0x69, 0x66, 0x31, // mif1
  ]);

  const isHeic = await detect(sampleData);
  console.log(`Is HEIC container: ${isHeic}`);

  try {
    const metadata = await inspect(sampleData, {
      maxFileSize: 10 * 1024 * 1024,
      maxWidth: 4096,
      maxHeight: 4096,
    });
    console.log('Metadata extracted:', metadata);
  } catch (err) {
    if (err instanceof LimitsExceededError) {
      console.error('Limit check failed:', err.message);
    } else {
      console.error('Inspection failed:', err);
    }
  }
}

main().catch(console.error);
