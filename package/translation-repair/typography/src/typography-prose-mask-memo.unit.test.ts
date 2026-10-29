/**
 Tests that the prose-mask memory keys a reading by the page's definitions and the text as two separate keys, so no text can
 read another text's mask (n160 finding Y9).

 The key was the definitions, a NUL character and the text joined in one string, so the text `[x]: u<NUL>See [dogs' bowl][x]` with
 no definitions shared the entry of `See [dogs' bowl][x]` with the definitions `[x]: u`, and a reader of the first text was
 handed a mask of the second's length.

 Cat-themed invention throughout.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { proseMaskReading, } from '../dist/final/node/index.mjs';

/**
 A paragraph whose reference label is prose only when no definition resolves it.
 */
const PARAGRAPH = 'See [dogs\' bowl][x] now.';

/**
 The definition the paragraph's reference resolves against.
 */
const DEFINITIONS = '[x]: u';

await describe({
  name: 'proseMaskReading memory',
  concurrency: 1,
  children: [
    it({
      name: 'KEEPS a text that holds the NUL and the definitions of another text apart from that other text',
      fn: async () => {
        /**
         The reading remembered first: the paragraph with its definitions.
         */
        const first = proseMaskReading({ text: PARAGRAPH, definitions: DEFINITIONS, },);
        /**
         A text built to join to the same single string as the paragraph with its definitions, read with none.
         */
        const forged = `${DEFINITIONS}\u0000${PARAGRAPH}`;
        /**
         The reading of the forged text.
         */
        const second = proseMaskReading({ text: forged, },);
        expect(first.mask.length,).toBe(PARAGRAPH.length,);
        expect(second.mask.length,).toBe(forged.length,);
        expect(second,).not.toBe(first,);
      },
    },),

    it({
      name: 'ANSWERS a repeated read of the same text and definitions with the reading it remembered',
      fn: async () => {
        /**
         The first read.
         */
        const first = proseMaskReading({ text: PARAGRAPH, definitions: DEFINITIONS, },);
        /**
         The second read of the same pair.
         */
        const again = proseMaskReading({ text: PARAGRAPH, definitions: DEFINITIONS, },);
        expect(again,).toBe(first,);
      },
    },),

    it({
      name: 'READS the same text differently with and without the definitions that resolve its reference',
      fn: async () => {
        /**
         The text read with its definitions.
         */
        const resolved = proseMaskReading({ text: PARAGRAPH, definitions: DEFINITIONS, },);
        /**
         The text read alone.
         */
        const alone = proseMaskReading({ text: PARAGRAPH, },);
        expect(resolved,).not.toBe(alone,);
        expect(resolved.mask,).not.toEqual(alone.mask,);
      },
    },),
  ],
},);
