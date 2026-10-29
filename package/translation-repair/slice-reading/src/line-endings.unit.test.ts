/**
 Tests for folding Windows line endings.

 WHAT THESE PIN is that the fold counts what it changed, leaves LF text and
 a lone carriage return alone, and shrinks the text by exactly the count.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { foldCarriageReturns, } from '../dist/final/node/index.mjs';

await describe({
  name: foldCarriageReturns.name,
  children: [
    it({
      name: 'folds every CRLF to LF and counts them, shrinking the text by exactly that many units',
      fn: async () => {
        /**
         A CRLF page.
         */
        const text = '猫猫喜欢晒太阳。\r\n\r\n它在窗台上睡觉。\r\n';

        /**
         What the fold made of it.
         */
        const { text: folded, folded: count, } = foldCarriageReturns({ text, },);
        expect(folded,).toBe('猫猫喜欢晒太阳。\n\n它在窗台上睡觉。\n',);
        expect(count,).toBe(3,);
        expect(text.length - folded.length,).toBe(count,);
      },
    },),
    it({
      name: 'leaves LF text unchanged and counts nothing, which is every other page in the pinned corpus',
      fn: async () => {
        /**
         An LF page.
         */
        const text = '猫猫喜欢晒太阳。\n\n它在窗台上睡觉。\n';
        expect(foldCarriageReturns({ text, },),).toEqual({
          text,
          folded: 0,
        },);
      },
    },),
    it({
      name: 'leaves a lone carriage return alone, since it is not a line ending this corpus writes and '
        + 'folding it would be a guess about text nobody has measured',
      fn: async () => {
        /**
         A stray return inside a line.
         */
        const text = '猫猫\r喜欢晒太阳。\n';
        expect(foldCarriageReturns({ text, },),).toEqual({
          text,
          folded: 0,
        },);
      },
    },),
  ],
},);
