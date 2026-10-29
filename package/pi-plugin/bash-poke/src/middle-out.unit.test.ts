/**
 Tests for middle-out assembly in the built bash-poke artifact.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  elisionMarker,
  middleOutFromParts,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    //region elisionMarker

    describe({
      name: elisionMarker.name,
      children: [
        it({
          name: 'names the elided character count between ellipses',
          fn: async () => {
            expect(elisionMarker({ elidedChars: 42, }, ), ).toBe('\u2026[42 characters elided]\u2026');
          },
        }, ),
        it({
          name: 'reports zero elision without inventing a count',
          fn: async () => {
            expect(elisionMarker({ elidedChars: 0, }, ), ).toContain('[0 characters elided]');
          },
        }, ),
      ],
    }, ),

    //endregion elisionMarker

    //region middleOutFromParts

    describe({
      name: middleOutFromParts.name,
      children: [
        it({
          name: 'joins head and tail around the marker',
          fn: async () => {
            const assembled = middleOutFromParts({
              head: 'ab',
              tail: 'ij',
              elidedChars: 6,
            }, );
            expect(assembled.split('\n', ), ).toHaveLength(3);
            expect(assembled.startsWith('ab\n', ), ).toBe(true);
            expect(assembled.endsWith('\nij', ), ).toBe(true);
            expect(assembled, ).toContain('[6 characters elided]');
          },
        }, ),
        it({
          name: 'omits an empty head without leaving a blank line',
          fn: async () => {
            const assembled = middleOutFromParts({
              head: '',
              tail: 'ij',
              elidedChars: 6,
            }, );
            expect(assembled.split('\n', ), ).toHaveLength(2);
            expect(assembled.endsWith('\nij', ), ).toBe(true);
          },
        }, ),
        it({
          name: 'omits an empty tail without leaving a blank line',
          fn: async () => {
            const assembled = middleOutFromParts({
              head: 'ab',
              tail: '',
              elidedChars: 6,
            }, );
            expect(assembled.split('\n', ), ).toHaveLength(2);
            expect(assembled.startsWith('ab\n', ), ).toBe(true);
          },
        }, ),
        it({
          name: 'returns only the marker when both budgets are zero',
          fn: async () => {
            const assembled = middleOutFromParts({
              head: '',
              tail: '',
              elidedChars: 9,
            }, );
            expect(assembled, ).toBe('\u2026[9 characters elided]\u2026');
          },
        }, ),
      ],
    }, ),

    //endregion middleOutFromParts
  ],
}, );
