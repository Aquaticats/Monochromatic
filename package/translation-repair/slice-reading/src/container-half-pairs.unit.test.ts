/**
 Tests the reader that names which two slices own one container's halves
 (class fifty-seven, XingZ607, 2026-09-18). Cat-themed invention throughout;
 no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChunkPair,
  containerHalfPairs,
} from '../dist/final/node/index.mjs';

/**
 Builds one prepared slice from its source text.

 @param sliceIndex - index the slice carries on both sides

 @param text - source text of the slice

 @param insertion - whether the archive carries nothing for it

 @returns Slice whose offsets are placeholders, since the reader reads text alone

 @example
 ```ts
 const slice = sliceOf({ sliceIndex: 0, text: '<details>', insertion: true, },);
 ```
 */
function sliceOf(
  {
    sliceIndex,
    text,
    insertion,
  }: {
    readonly sliceIndex: number;
    readonly text: string;
    readonly insertion: boolean;
  },
): ChunkPair {
  return {
    source: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: text.length,
      text,
    },
    target: insertion
      ? makeInsertionChunk({ sliceIndex, offset: 0, },)
      : {
        kind: 'content',
        sliceIndex,
        nodes: [],
        startOffset: 0,
        endOffset: 4,
        text: 'Cat.',
      },
  };
}

/**
 The pair a container opened in the first of two archive-carried slices and
 closed in the second makes.
 */
const FIRST_TO_SECOND = {
  open: {
    sliceIndex: 0,
    position: 0,
    insertion: false,
    name: 'details',
  },
  close: {
    sliceIndex: 1,
    position: 1,
    insertion: false,
    name: 'details',
  },
};

await describe({
  name: containerHalfPairs.name,
  children: [
    it({
      name: 'PAIRS the slice opening a block with the slice closing it, across the slices between them',
      fn: async () => {
        const pairs = containerHalfPairs({
          slices: [
            sliceOf({ sliceIndex: 0, text: '猫猫在窗台上打盹。', insertion: false, },),
            sliceOf({ sliceIndex: 1, text: '<details>\n<summary>猫的故事</summary>', insertion: true, },),
            sliceOf({ sliceIndex: 2, text: '猫在花园里追蝴蝶。', insertion: true, },),
            sliceOf({ sliceIndex: 3, text: '猫在夜里回家了。\n\n</details>', insertion: true, },),
          ],
        },);
        expect(pairs,).toEqual([{
          open: { sliceIndex: 1, position: 1, insertion: true, name: 'details', },
          close: { sliceIndex: 3, position: 3, insertion: true, name: 'details', },
        },],);
      },
    },),
    it({
      name: 'PAIRS nested blocks of one name innermost first, as the grammar closes them',
      fn: async () => {
        const pairs = containerHalfPairs({
          slices: [
            sliceOf({ sliceIndex: 0, text: '<details>\n<summary>外层</summary>', insertion: true, },),
            sliceOf({ sliceIndex: 1, text: '<details>\n<summary>内层</summary>', insertion: true, },),
            sliceOf({ sliceIndex: 2, text: '猫。\n\n</details>', insertion: true, },),
            sliceOf({ sliceIndex: 3, text: '狗。\n\n</details>', insertion: false, },),
          ],
        },);
        expect(pairs.map(function toIndices(pair,): readonly number[] {
          return [
            pair.open
              .sliceIndex,
            pair.close
              .sliceIndex,
          ];
        },),).toEqual([
          [1, 2,],
          [0, 3,],
        ],);
        expect(pairs[1]?.close
          .insertion,).toBe(false,);
      },
    },),
    it({
      name: 'LEAVES a closing tag with no opening, and a block whole inside one slice, out of the pairs',
      fn: async () => {
        const pairs = containerHalfPairs({
          slices: [
            sliceOf({ sliceIndex: 0, text: '猫。\n\n</details>', insertion: true, },),
            sliceOf({ sliceIndex: 1, text: '<details>\n<summary>猫</summary>\n\n喵。\n\n</details>', insertion: true, },),
          ],
        },);
        expect(pairs,).toEqual([],);
      },
    },),
    it({
      name: 'READS NO TAG out of a name running on into a word or a self-closing tag, so the closing tag line '
        + 'beside them stays lone in its slice and pairs with the opening half of the slice before',
      fn: async () => {
        const pairs = containerHalfPairs({
          slices: [
            sliceOf({ sliceIndex: 0, text: '<details>\n猫猫在窗台上睡觉。', insertion: false, },),
            sliceOf({
              sliceIndex: 1,
              text: '<detailsx>猫猫追蝴蝶。\n<details/>猫猫伸了个懒腰。\n</details>',
              insertion: false,
            },),
          ],
        },);
        expect(pairs,).toEqual([FIRST_TO_SECOND,],);
      },
    },),
    it({
      name: 'READS NO TAG out of a closer never closed beside prose, so the opening tag line of its slice '
        + 'stays lone and pairs with the closing half of the slice after',
      fn: async () => {
        const pairs = containerHalfPairs({
          slices: [
            sliceOf({ sliceIndex: 0, text: '<details>\n猫猫在窗台上睡觉。</details', insertion: false, },),
            sliceOf({ sliceIndex: 1, text: '猫猫追蝴蝶。\n</details>', insertion: false, },),
          ],
        },);
        expect(pairs,).toEqual([FIRST_TO_SECOND,],);
      },
    },),
    it({
      name: 'READS a brace close at no brace depth as text rather than a close, so its tag still ends at the '
        + '`>` after it and takes the closing half as its partner',
      fn: async () => {
        const pairs = containerHalfPairs({
          slices: [
            sliceOf({ sliceIndex: 0, text: '<details>\n猫猫在窗台上睡觉。', insertion: false, },),
            sliceOf({
              sliceIndex: 1,
              text: '<details data-x=}>猫猫追蝴蝶。\n</details>',
              insertion: false,
            },),
          ],
        },);
        expect(pairs,).toEqual([],);
      },
    },),
    it({
      name: 'READS a closer inline beside prose as the partner of its slice\'s opening tag line, so the '
        + 'closing half of the slice after finds none',
      fn: async () => {
        const pairs = containerHalfPairs({
          slices: [
            sliceOf({ sliceIndex: 0, text: '<details>\n猫猫追蝴蝶。</details>', insertion: false, },),
            sliceOf({ sliceIndex: 1, text: '猫猫伸了个懒腰。\n</details>', insertion: false, },),
          ],
        },);
        expect(pairs,).toEqual([],);
      },
    },),
    it({
      name: 'READS an opener inline beside prose as the partner of its slice\'s closing tag line, so the '
        + 'opening tag line before them pairs with the closing half of the slice after',
      fn: async () => {
        const pairs = containerHalfPairs({
          slices: [
            sliceOf({
              sliceIndex: 0,
              text: '<details>\n猫猫在窗台上睡觉。<details>\n</details>',
              insertion: false,
            },),
            sliceOf({ sliceIndex: 1, text: '猫猫追蝴蝶。\n</details>', insertion: false, },),
          ],
        },);
        expect(pairs,).toEqual([FIRST_TO_SECOND,],);
      },
    },),
  ],
},);
