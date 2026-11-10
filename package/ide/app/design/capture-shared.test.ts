// Checks of the capture helpers the two design scripts share.
// What: the PNG flat-frame probe, the settled-screenshot loop, and the session lifecycle, exercised without the real
//       compositor: a screenshot is a real file the fake control socket writes, and a stand-in compositor ends the way
//       a compositor that cannot start its client does.
// Why: the settle rule decides which frame a capture script records, and a session that ended must be reported at once
//      with what it said, because a capture run that waits the readiness deadline out reports nothing useful.
import assert from 'node:assert/strict';
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join, resolve } from 'node:path';
import { test } from 'node:test';
import { deflateSync } from 'node:zlib';
import { READY_DEADLINE_MS, SETTLE_DRAWN_ATTEMPTS, captureSession, flat, sessionEnvironment, settleScreenshot, waitForCompositor } from './capture-shared.ts';

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
    await waitForCompositor({
      deadline: Date.now() + READY_DEADLINE_MS,
      control: async () => {
        asked += 1;
        return asked < 2 ? 'busy' : 'ok';
      },
    });
    assert.equal(asked, 2, 'a compositor that is not up yet is asked again');
  }
);

test(
  'a compositor that died at startup is reported with its socket error',
  async () => {
    // A compositor killed while it starts closes its socket instead of answering, which is not a slow compositor.
    await assert.rejects(
      waitForCompositor({
        deadline: Date.now(),
        control: async () => {
          throw new Error('connect ENOENT /captures/control.sock');
        },
      }),
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

// A stand-in compositor that ends at startup, the way a compositor that cannot start its client does.
// What: an executable program that records the arguments it was handed, prints one line, and exits with a failure.
// Why: the session's own report has to arrive at once and name where the compositor's words were written, and the
//      application path the caller gave has to reach the compositor from a session that runs in another folder.
const dyingCompositor = ({ directory, line }: { directory: string; line: string }): string => {
  const path = join(directory, 'compositor-that-ends-at-startup.cjs');
  writeFileSync(path, [
    '#!/usr/bin/env node',
    '// Records the arguments the session handed it, prints one line, and ends with a failure.',
    'const { writeFileSync } = require("node:fs");',
    'const { join } = require("node:path");',
    'writeFileSync(join(__dirname, "arguments.json"), JSON.stringify(process.argv.slice(2)));',
    'process.stderr.write(' + JSON.stringify(line + '\n') + ');',
    'process.exit(1);',
    '',
  ].join('\n'));
  chmodSync(path, 0o755);
  return path;
};

// One session of the stand-in compositor over an empty project, started the way a capture script starts a session.
const captureOver = ({ directory, compositor }: { directory: string; compositor: string }): Promise<void> => captureSession({
  application: 'target/debug/monochromatic-ide',
  project: join(directory, 'project'),
  file: 'sample.rs',
  scheme: 'dark',
  size: '1100x660',
  directory,
  env: sessionEnvironment({ directory }),
  logPath: join(directory, 'session.log'),
  compositor,
}, async () => {
  throw new Error('a session whose compositor ended at startup must not run the requests of its script');
});

test(
  'the application reaches the compositor as a path from the root, so a caller may name it from its own folder',
  async () => {
    const directory = scratch();
    try {
      mkdirSync(join(directory, 'project'), { recursive: true });
      const compositor = dyingCompositor({ directory, line: 'failed to spawn hosted client' });
      await captureOver({ directory, compositor }).catch(() => undefined);
      const handed: string[] = JSON.parse(readFileSync(join(directory, 'arguments.json'), 'utf8'));
      const hosted = handed[handed.indexOf('--') + 1];
      assert.equal(hosted, resolve('target/debug/monochromatic-ide'), 'the application the caller named from its own folder is started by its path from the root');
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  }
);

test(
  'a compositor that ended before answering is reported at once, with its socket failure and where it said why',
  async () => {
    const directory = scratch();
    try {
      mkdirSync(join(directory, 'project'), { recursive: true });
      const said = 'failed to spawn hosted client: target/debug/monochromatic-ide';
      const compositor = dyingCompositor({ directory, line: said });
      const started = performance.now();
      await assert.rejects(captureOver({ directory, compositor }), { message: /ended with status 1[\s\S]*session\.log/ });
      const waited = Math.round(performance.now() - started);
      assert.ok(waited < 5000, 'the session reports the end of its compositor instead of waiting the readiness deadline out; it waited ' + waited + ' ms');
      assert.match(readFileSync(join(directory, 'session.log'), 'utf8'), new RegExp(said), 'the reported log is the one that holds why the compositor ended');
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  }
);
