/**
 Tests the one span-rewrite applier the page passes share (audit area six):
 rewrites applied in offset order whatever order they arrive in, and one that
 overlaps a rewrite kept before it withheld rather than spliced behind the
 cursor, which repeated text on the page.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  applySpanRewrites,
  correctPinyinTones,
} from '../../dist/final/node/index.mjs';

/**
 Text the cases rewrite.
 */
const TEXT = 'the grey cat naps';

await describe({
  name: applySpanRewrites.name,
  children: [
    it({
      name: 'APPLIES REWRITES IN OFFSET ORDER whatever order they arrive in, adjacent spans included',
      fn: async () => {
        /**
         Rewrites handed over last first.
         */
        const rebuilt = applySpanRewrites({
          text: TEXT,
          rewrites: [
            { start: 13, end: 17, to: 'dozes', },
            { start: 9, end: 12, to: 'tabby', },
            { start: 4, end: 8, to: 'gray', },
            { start: 8, end: 9, to: '-', },
          ],
        },);
        expect(rebuilt.text,).toBe('the gray-tabby dozes',);
        expect(rebuilt.applied.map(function startOf(rewrite,): number {
          return rewrite.start;
        },),).toEqual([4, 8, 9, 13,],);
      },
    },),
    it({
      name: 'WITHHOLDS A REWRITE OVERLAPPING ONE KEPT BEFORE IT, the longer kept where two start together, '
        + 'so no text is repeated',
      fn: async () => {
        /**
         Rewrites where a shorter one starts with a longer, and another sits
         inside the longer.
         */
        const rebuilt = applySpanRewrites({
          text: TEXT,
          rewrites: [
            { start: 4, end: 8, to: 'gray', },
            { start: 4, end: 12, to: 'gray cat', },
            { start: 9, end: 12, to: 'kitten', },
          ],
        },);
        expect(rebuilt.text,).toBe('the gray cat naps',);
        expect(rebuilt.applied,).toEqual([{ start: 4, end: 12, to: 'gray cat', },],);
      },
    },),
    it({
      name: 'LEAVES A TEXT WITH NO REWRITES as it is',
      fn: async () => {
        expect(applySpanRewrites({ text: TEXT, rewrites: [], },),).toEqual({ text: TEXT, applied: [], },);
      },
    },),
    it({
      name: 'THE PINYIN PASS, which read rewrites in parenthesis order, corrects two syllables in one text once each',
      fn: async () => {
        expect(correctPinyinTones({ text: '(线, xiǎn) and (线, xiǎn)', },).text,).toBe('(线, xiàn) and (线, xiàn)',);
      },
    },),
  ],
},);
