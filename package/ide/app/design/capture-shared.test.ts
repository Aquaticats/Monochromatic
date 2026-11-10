// Checks of the capture helpers the two design scripts share.
// What: the PNG flat-frame probe and the settled-screenshot loop, exercised without a compositor: a screenshot is a
//       real file the fake control socket writes.
// Why: the settle rule decides which frame a capture script records, and both scripts now record through it.
import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { deflateSync } from 'node:zlib';
import { READY_DEADLINE_MS, SETTLE_DRAWN_ATTEMPTS, flat, settleScreenshot, waitForCompositor } from './capture-shared.ts';

// A CRC32 of the chunk bytes, so the fixture is a PNG a decoder accepts rather than only the bytes `flat` reads.
const crcTable: number[] = [];
for (let index = 0; index < 256; index++) {
  let value = index;
  for (let bit = 0; bit < 8; bit++) value = (value & 1) === 1 ? 0xEDB88320 ^ (value >>> 1) : value >>> 1;
  crcTable.push(value >>> 0);
}
const crc32 = (bytes: Buffer): number => {
  let crc = 0xFFFFFFFF;
  for (const byte of bytes) crc = (crcTable[(crc ^ byte) & 0xFF] ?? 0) ^ (crc >>> 8);
  return (crc ^ 0xFFFFFFFF) >>> 0;
};
const chunk = (type: string, data: Buffer): Buffer => {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length);
  const typed = Buffer.concat([Buffer.from(type, 'latin1'), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(typed));
  return Buffer.concat([length, typed, crc]);
};

// One 8-bit RGBA image whose pixel at (row, column) is `pixel(row, column)`.
const png = (size: number, pixel: (row: number, column: number) => [number, number, number, number]): Buffer => {
  const header = Buffer.alloc(13);
  header.writeUInt32BE(size, 0);
  header.writeUInt32BE(size, 4);
  header[8] = 8;
  header[9] = 6;
  const raw: number[] = [];
  for (let row = 0; row < size; row++) {
    raw.push(0);
    for (let column = 0; column < size; column++) raw.push(...pixel(row, column));
  }
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
    chunk('IHDR', header),
    chunk('IDAT', deflateSync(Buffer.from(raw))),
    chunk('IEND', Buffer.alloc(0)),
  ]);
};

// A disposable directory below the private agent scratch root, removed with the test that made it.
const scratch = (): string => {
  const root = join(homedir(), 'temp', 'agent');
  mkdirSync(root, { recursive: true });
  return mkdtempSync(join(root, 'ide-capture-shared-'));
};

test(
  'the readiness wait returns once the compositor answers ok',
  async () => {
    let asked = 0;
    await waitForCompositor(
      async () => {
        asked += 1;
        return asked < 2 ? 'busy' : 'ok';
      },
      Date.now() + READY_DEADLINE_MS
    );
    assert.equal(asked, 2, 'a compositor that is not up yet is asked again');
  }
);

test(
  'a compositor that died at startup is reported with its socket error',
  async () => {
    // A compositor killed while it starts closes its socket instead of answering, which is not a slow compositor.
    await assert.rejects(
      waitForCompositor(async () => {
        throw new Error('connect ENOENT /captures/control.sock');
      }, Date.now()),
      { message: /connect ENOENT \/captures\/control.sock/ }
    );
  }
);

test(
  'an undrawn window is one flat color',
  () => {
    assert.equal(
      flat(png(12, () => [0, 0, 0, 255])),
      true,
      'a single color is what an undrawn window looks like'
    );
  }
);

test(
  'a drawn window holds more sampled colors than a flat one',
  () => {
    // The probe samples one pixel per row here, so each row carries its own color.
    assert.equal(
      flat(png(12, (row) => [row * 20, 0, 0, 255])),
      false,
      'twelve sampled colors are not a flat frame'
    );
  }
);

test(
  'the settled-frame loop returns once two screenshots agree',
  async () => {
    const directory = scratch();
    try {
      const path = join(directory, 'frame.png');
      const drawn = png(12, (row) => [row * 20, 0, 0, 255]);
      let taken = 0;
      await settleScreenshot({
        path,
        attempts: 40,
        intervalMs: 0,
        control: async (line) => {
          assert.equal(line, 'screenshot ' + path);
          taken += 1;
          // The first draws are transitional, then the frame holds.
          writeFileSync(path, taken < 3 ? Buffer.concat([drawn, Buffer.from([taken])]) : drawn);
          return 'ok';
        },
      });
      // A frame may still be the first draw for `SETTLE_DRAWN_ATTEMPTS` screenshots, so the first pair that counts
      // is the one after that, which is the next screenshot.
      assert.equal(taken, SETTLE_DRAWN_ATTEMPTS + 2, 'the loop stops at the first pair that agrees after the first draws');
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  }
);

test(
  'a frame that never settles fails the loop instead of recording a transitional screenshot',
  async () => {
    const directory = scratch();
    try {
      const path = join(directory, 'frame.png');
      const drawn = png(12, (row) => [row * 20, 0, 0, 255]);
      let taken = 0;
      await assert.rejects(
        settleScreenshot({
          path,
          attempts: 6,
          intervalMs: 0,
          control: async () => {
            taken += 1;
            writeFileSync(path, Buffer.concat([drawn, Buffer.from([taken])]));
            return 'ok';
          },
        }),
        { message: 'frame never settled: ' + path }
      );
      assert.equal(taken, 6, 'every allowed attempt is spent before the loop gives up');
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  }
);
