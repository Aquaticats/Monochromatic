/**
 Tests for bounded output recording in the built bash-poke artifact.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createOutputRecorder,
  trimRollingBuffer,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    //region trimRollingBuffer

    describe({
      name: trimRollingBuffer.name,
      children: [
        it({
          name: 'leaves a buffer inside its budget untouched',
          fn: async () => {
            const chunks = ['ab', 'cd', ];
            expect(trimRollingBuffer({ chunks, chars: 4, budget: 10, }, ), ).toBe(4);
            expect(chunks, ).toEqual(['ab', 'cd', ]);
          },
        }, ),
        it({
          name: 'drops whole leading chunks down to the budget',
          fn: async () => {
            const chunks = ['ab', 'cd', 'ef', ];
            expect(trimRollingBuffer({ chunks, chars: 6, budget: 4, }, ), ).toBe(4);
            expect(chunks, ).toEqual(['cd', 'ef', ]);
          },
        }, ),
        it({
          name: 'cuts the boundary chunk instead of dropping it whole',
          fn: async () => {
            const chunks = ['abcd', 'ef', ];
            expect(trimRollingBuffer({ chunks, chars: 6, budget: 3, }, ), ).toBe(3);
            expect(chunks, ).toEqual(['d', 'ef', ]);
          },
        }, ),
        it({
          name: 'empties the buffer for a zero budget',
          fn: async () => {
            const chunks = ['ab', ];
            expect(trimRollingBuffer({ chunks, chars: 2, budget: 0, }, ), ).toBe(0);
            expect(chunks, ).toEqual([]);
          },
        }, ),
        it({
          name: 'reports the given total when there is nothing to drop',
          fn: async () => {
            const chunks: string[] = [];
            expect(trimRollingBuffer({ chunks, chars: 5, budget: 2, }, ), ).toBe(5);
          },
        }, ),
      ],
    }, ),

    //endregion trimRollingBuffer

    //region createOutputRecorder

    describe({
      name: createOutputRecorder.name,
      children: [
        it({
          name: 'reproduces output that fits the budget exactly',
          fn: async () => {
            const recorder = createOutputRecorder({ headChars: 10, tailChars: 10, }, );
            recorder.append('abc', );
            const snapshot = recorder.snapshot();
            expect(snapshot.text, ).toBe('abc');
            expect(snapshot.truncated, ).toBe(false);
            expect(snapshot.elidedChars, ).toBe(0);
            expect(snapshot.totalChars, ).toBe(3);
          },
        }, ),
        it({
          name: 'ignores an empty chunk',
          fn: async () => {
            const recorder = createOutputRecorder({ headChars: 4, tailChars: 4, }, );
            recorder.append('', );
            expect(recorder.snapshot().totalChars, ).toBe(0);
            expect(recorder.snapshot().text, ).toBe('');
          },
        }, ),
        it({
          name: 'keeps head and tail and counts the elided middle',
          fn: async () => {
            const recorder = createOutputRecorder({ headChars: 2, tailChars: 3, }, );
            recorder.append('abcdefghij', );
            const snapshot = recorder.snapshot();
            expect(snapshot.truncated, ).toBe(true);
            expect(snapshot.elidedChars, ).toBe(5);
            expect(snapshot.totalChars, ).toBe(10);
            expect(snapshot.text.startsWith('ab\n', ), ).toBe(true);
            expect(snapshot.text.endsWith('\nhij', ), ).toBe(true);
            expect(snapshot.text, ).toContain('[5 characters elided]');
          },
        }, ),
        it({
          name: 'accumulates across many chunks',
          fn: async () => {
            const recorder = createOutputRecorder({ headChars: 2, tailChars: 2, }, );
            recorder.append('ab', );
            recorder.append('cd', );
            recorder.append('ef', );
            const snapshot = recorder.snapshot();
            expect(snapshot.totalChars, ).toBe(6);
            expect(snapshot.truncated, ).toBe(true);
            expect(snapshot.elidedChars, ).toBe(2);
            expect(snapshot.text.startsWith('ab\n', ), ).toBe(true);
            expect(snapshot.text.endsWith('\nef', ), ).toBe(true);
          },
        }, ),
        it({
          name: 'keeps only the tail when the head budget is zero',
          fn: async () => {
            const recorder = createOutputRecorder({ headChars: 0, tailChars: 3, }, );
            recorder.append('abcdef', );
            const snapshot = recorder.snapshot();
            expect(snapshot.truncated, ).toBe(true);
            expect(snapshot.elidedChars, ).toBe(3);
            expect(snapshot.text.endsWith('\ndef', ), ).toBe(true);
            expect(snapshot.text.startsWith('\u2026', ), ).toBe(true);
          },
        }, ),
        it({
          name: 'keeps only the head when the tail budget is zero',
          fn: async () => {
            const recorder = createOutputRecorder({ headChars: 3, tailChars: 0, }, );
            recorder.append('abcdef', );
            const snapshot = recorder.snapshot();
            expect(snapshot.truncated, ).toBe(true);
            expect(snapshot.elidedChars, ).toBe(3);
            expect(snapshot.text.startsWith('abc\n', ), ).toBe(true);
          },
        }, ),
        it({
          name: 'reports only the marker when both budgets are zero',
          fn: async () => {
            const recorder = createOutputRecorder({ headChars: 0, tailChars: 0, }, );
            recorder.append('abc', );
            const snapshot = recorder.snapshot();
            expect(snapshot.truncated, ).toBe(true);
            expect(snapshot.elidedChars, ).toBe(3);
            expect(snapshot.text, ).toContain('[3 characters elided]');
          },
        }, ),
      ],
    }, ),

    describe({
      name: 'createOutputRecorder.tailLines',
      children: [
        it({
          name: 'returns the most recent lines',
          fn: async () => {
            const recorder = createOutputRecorder({ headChars: 100, tailChars: 100, }, );
            recorder.append('l1\nl2\nl3\nl4', );
            expect(recorder.tailLines(2, ), ).toEqual(['l3', 'l4', ]);
          },
        }, ),
        it({
          name: 'returns nothing for a zero line count',
          fn: async () => {
            const recorder = createOutputRecorder({ headChars: 100, tailChars: 100, }, );
            recorder.append('l1\nl2', );
            expect(recorder.tailLines(0, ), ).toEqual([]);
          },
        }, ),
        it({
          name: 'returns everything when the count exceeds the lines held',
          fn: async () => {
            const recorder = createOutputRecorder({ headChars: 100, tailChars: 100, }, );
            recorder.append('l1\nl2', );
            expect(recorder.tailLines(9, ), ).toEqual(['l1', 'l2', ]);
          },
        }, ),
        it({
          name: 'returns nothing before any output arrives',
          fn: async () => {
            const recorder = createOutputRecorder({ headChars: 10, tailChars: 10, }, );
            expect(recorder.tailLines(3, ), ).toEqual([]);
          },
        }, ),
      ],
    }, ),

    //endregion createOutputRecorder
  ],
}, );
