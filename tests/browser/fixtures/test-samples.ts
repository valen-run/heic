/**
 * Deterministic synthetic HEIC test samples for browser environments.
 */

function makeBox(fourcc: string, payload: Uint8Array): Uint8Array {
  const size = 8 + payload.length;
  const buf = new Uint8Array(size);
  const view = new DataView(buf.buffer);
  view.setUint32(0, size, false);
  for (let i = 0; i < 4; i++) {
    buf[4 + i] = fourcc.charCodeAt(i);
  }
  buf.set(payload, 8);
  return buf;
}

function makeFullBox(fourcc: string, version: number, flags: number, payload: Uint8Array): Uint8Array {
  const size = 12 + payload.length;
  const buf = new Uint8Array(size);
  const view = new DataView(buf.buffer);
  view.setUint32(0, size, false);
  for (let i = 0; i < 4; i++) {
    buf[4 + i] = fourcc.charCodeAt(i);
  }
  buf[8] = version;
  buf[9] = (flags >> 16) & 0xff;
  buf[10] = (flags >> 8) & 0xff;
  buf[11] = flags & 0xff;
  buf.set(payload, 12);
  return buf;
}

function concat(arrays: Uint8Array[]): Uint8Array {
  const total = arrays.reduce((acc, a) => acc + a.length, 0);
  const result = new Uint8Array(total);
  let offset = 0;
  for (const arr of arrays) {
    result.set(arr, offset);
    offset += arr.length;
  }
  return result;
}

/**
 * Builds a minimal valid synthetic HEIC container (1920x1080).
 */
export function createSyntheticHeic(width = 1920, height = 1080): Uint8Array {
  // ftyp
  const ftypPayload = concat([
    new TextEncoder().encode('heic'),
    new Uint8Array(4),
    new TextEncoder().encode('mif1'),
  ]);
  const ftyp = makeBox('ftyp', ftypPayload);

  // meta
  const hdlr = makeFullBox('hdlr', 0, 0, concat([new Uint8Array(4), new TextEncoder().encode('pict'), new Uint8Array(12)]));
  const pitm = makeFullBox('pitm', 0, 0, new Uint8Array([0, 1]));

  const infePayload = concat([
    new Uint8Array([0, 1, 0, 0]),
    new TextEncoder().encode('hvc1'),
    new Uint8Array([0]),
  ]);
  const infe = makeFullBox('infe', 2, 0, infePayload);
  const iinf = makeFullBox('iinf', 0, 0, concat([new Uint8Array([0, 1]), infe]));

  const ispePayload = new Uint8Array(8);
  const ispeView = new DataView(ispePayload.buffer);
  ispeView.setUint32(0, width, false);
  ispeView.setUint32(4, height, false);
  const ispe = makeFullBox('ispe', 0, 0, ispePayload);
  const ipco = makeBox('ipco', ispe);

  const ipma = makeFullBox('ipma', 0, 0, new Uint8Array([0, 0, 0, 1, 0, 1, 1, 1]));
  const iprp = makeBox('iprp', concat([ipco, ipma]));

  // mdat payload
  const mdatPayload = new Uint8Array([
    0x00, 0x00, 0x00, 0x01, 0x40, 0x01, 0x0c, // VPS
    0x00, 0x00, 0x00, 0x01, 0x42, 0x01, 0x01, // SPS
    0x00, 0x00, 0x00, 0x01, 0x44, 0x01, 0xc0, // PPS
    0x00, 0x00, 0x00, 0x01, 0x26, 0x01, 0xaf, 0x55, 0x55, // Slice
  ]);

  const ilocStub = makeFullBox('iloc', 0, 0, new Uint8Array(16));
  const metaTotalSize = 12 + hdlr.length + pitm.length + iinf.length + iprp.length + ilocStub.length;
  const mdatOffset = ftyp.length + metaTotalSize + 8;

  const ilocPayload = new Uint8Array(16);
  const ilocView = new DataView(ilocPayload.buffer);
  ilocPayload[0] = 0x44;
  ilocPayload[1] = 0x00;
  ilocView.setUint16(2, 1, false); // 1 item
  ilocView.setUint16(4, 1, false); // item 1
  ilocView.setUint16(6, 0, false); // dref 0
  ilocView.setUint16(8, 1, false); // 1 extent
  ilocView.setUint32(10, mdatOffset, false);
  ilocView.setUint32(14, mdatPayload.length, false);
  const iloc = makeFullBox('iloc', 0, 0, ilocPayload);

  const meta = makeFullBox('meta', 0, 0, concat([hdlr, pitm, iinf, iprp, iloc]));
  const mdat = makeBox('mdat', mdatPayload);

  return concat([ftyp, meta, mdat]);
}
