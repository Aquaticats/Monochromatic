// Checks of the frame-pattern analysis behind the IDE's cadence check.
// What: a frame hash follows the pixels and not the header, the input window follows the recorder rate, the counts and
// runs of identical frames are exact on hand-made patterns including the edges (a run that ends the window, a window
// past the recording, a window of one slot), and the printed lines carry every number the check reports.
// Why: the check decides from these counts whether a build stalls, and a miscount (a trailing run dropped, the
// quiet frames before the input counted as stalls) would let a frozen pipeline pass or a smooth one fail.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { analyzeCadence, FrameFormatError, formatCadence, frameHash, inputWindow, recordFps } from './cadence.ts';

// A 24 bit BMP of the given pixel bytes: a 14 byte file header whose offset field says where the pixels begin, then
// a filler of `gap` bytes standing for the information header.
const bmp = ({ pixels, gap = 40, marker = 0 }: { pixels: readonly number[]; gap?: number; marker?: number }) => {
  const bytes = new Uint8Array(14 + gap + pixels.length);
  bytes[0] = 0x42;
  bytes[1] = 0x4D;
  new DataView(bytes.buffer).setUint32(10, 14 + gap, true);
  bytes.fill(marker, 14, 14 + gap);
  bytes.set(pixels, 14 + gap);
  return bytes;
};

// Hashes named by letters: equal letters are equal pictures.
const hashesOf = (letters: string) => letters.split('');

test('equal pixels hash alike whatever the header holds, and different pixels differ', () => {
  const first = frameHash(bmp({ pixels: [1, 2, 3, 4], marker: 0 }));
  assert.equal(first, frameHash(bmp({ pixels: [1, 2, 3, 4], marker: 9 })));
  assert.notEqual(first, frameHash(bmp({ pixels: [1, 2, 3, 5] })));
  assert.match(first, /^[0-9a-f]{40}$/);
});

test('the pixel offset field decides where the hash starts', () => {
  const near = bmp({ pixels: [7, 7], gap: 40 });
  const far = bmp({ pixels: [7, 7], gap: 120 });
  assert.equal(frameHash(near), frameHash(far));
});

test('a file that is not a BMP, or whose offset lies outside it, is refused with a message naming the problem', () => {
  assert.throws(() => frameHash(new Uint8Array([0x89, 0x50, 0x4E, 0x47, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])), FrameFormatError);
  assert.throws(() => frameHash(new Uint8Array(5)), FrameFormatError);
  const broken = bmp({ pixels: [1] });
  new DataView(broken.buffer).setUint32(10, broken.length + 1, true);
  assert.throws(() => frameHash(broken), /offset .* lies outside/);
  const inside = bmp({ pixels: [1] });
  new DataView(inside.buffer).setUint32(10, 3, true);
  assert.throws(() => frameHash(inside), FrameFormatError);
});

test('a view into a larger buffer hashes only its own bytes', () => {
  const frame = bmp({ pixels: [1, 2, 3] });
  const padded = new Uint8Array(frame.length + 10);
  padded.set(frame, 5);
  assert.equal(frameHash(padded.subarray(5, 5 + frame.length)), frameHash(frame));
});

test('the input window is the recorder rate times the time of the first and of the last moment', () => {
  assert.equal(recordFps, 60);
  assert.deepEqual(inputWindow({ offsetMs: 500, inputMs: 1_920 }), { firstSlot: 30, lastSlot: 145 });
  assert.deepEqual(inputWindow({ offsetMs: 0, inputMs: 0 }), { firstSlot: 0, lastSlot: 0 });
  assert.deepEqual(inputWindow({ offsetMs: 508, inputMs: 1_000 }), { firstSlot: 30, lastSlot: 90 });
});

test('every frame that changes counts as changed and no run is reported', () => {
  const result = analyzeCadence({ hashes: hashesOf('abcdef'), window: { firstSlot: 0, lastSlot: 5 } });
  assert.deepEqual([result.changed, result.identical, result.longestIdenticalRun, result.identicalRuns], [5, 0, 0, []]);
  assert.equal(result.total, 6);
  assert.equal(result.pattern, 'XXXXXX');
});

test('runs of identical frames are listed in order with the longest named', () => {
  const result = analyzeCadence({ hashes: hashesOf('abbcdddde'), window: { firstSlot: 0, lastSlot: 8 } });
  assert.deepEqual([result.changed, result.identical, result.longestIdenticalRun], [4, 4, 3]);
  assert.deepEqual(result.identicalRuns, [{ start: 2, length: 1 }, { start: 5, length: 3 }]);
  assert.equal(result.pattern, 'XX.XX...X');
});

test('a run that reaches the last slot of the window is listed', () => {
  const result = analyzeCadence({ hashes: hashesOf('abccc'), window: { firstSlot: 0, lastSlot: 4 } });
  assert.deepEqual(result.identicalRuns, [{ start: 3, length: 2 }]);
  assert.equal(result.longestIdenticalRun, 2);
});

test('frames outside the window are not judged, though the pattern shows them', () => {
  // Quiet frames before and after the input are identical and must not count as stalls.
  const result = analyzeCadence({ hashes: hashesOf('aaaabcdeeee'), window: { firstSlot: 3, lastSlot: 7 } });
  assert.deepEqual([result.changed, result.identical, result.identicalRuns], [4, 0, []]);
  assert.equal(result.pattern, 'X...XXXX...');
});

test('a window that runs past the recording stops at the last frame', () => {
  const result = analyzeCadence({ hashes: hashesOf('abb'), window: { firstSlot: 0, lastSlot: 50 } });
  assert.deepEqual([result.changed, result.identical, result.identicalRuns], [1, 1, [{ start: 2, length: 1 }]]);
});

test('a window of one slot, an empty recording, and a first slot of zero compare nothing outside the recording', () => {
  assert.deepEqual(analyzeCadence({ hashes: hashesOf('abc'), window: { firstSlot: 1, lastSlot: 1 } }).changed, 0);
  const empty = analyzeCadence({ hashes: [], window: { firstSlot: 0, lastSlot: 10 } });
  assert.deepEqual([empty.total, empty.changed, empty.identical, empty.pattern], [0, 0, 0, '']);
  assert.equal(analyzeCadence({ hashes: hashesOf('ab'), window: { firstSlot: -5, lastSlot: 1 } }).changed, 1);
});

test('a frozen pipeline shows as one long run and a smooth one as none', () => {
  const frozen = analyzeCadence({ hashes: hashesOf(`a${  'b'.repeat(30)  }c`), window: { firstSlot: 0, lastSlot: 31 } });
  assert.deepEqual([frozen.longestIdenticalRun, frozen.identicalRuns], [29, [{ start: 2, length: 29 }]]);
  const smooth = analyzeCadence({ hashes: Array.from({ length: 120 }, (_, slot) => String(slot)), window: { firstSlot: 0, lastSlot: 119 } });
  assert.deepEqual([smooth.changed, smooth.identical, smooth.identicalRuns], [119, 0, []]);
});

test('the printed lines carry the totals, the window, the longest run, the runs, and the bracketed pattern', () => {
  const result = analyzeCadence({ hashes: hashesOf('aabccd'), window: { firstSlot: 1, lastSlot: 4 } });
  assert.deepEqual(formatCadence(result), [
    'frames recorded: 6, input window: slots 1 to 4',
    'in window: 3 compared, 2 changed, 1 identical',
    'longest identical run: 1',
    'identical runs (length@first slot): [1@4]',
    'pattern (X changed, . identical): X[.XX.]X',
  ]);
});

test('a window of one slot is bracketed on a single frame', () => {
  const lines = formatCadence(analyzeCadence({ hashes: hashesOf('ab'), window: { firstSlot: 1, lastSlot: 1 } }));
  assert.equal(lines[4], 'pattern (X changed, . identical): X[X]');
});
