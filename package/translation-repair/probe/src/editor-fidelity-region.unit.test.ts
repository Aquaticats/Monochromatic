/**
 Tests for the region an inserted sentence is quoted in: the sentence with the one either side of it on its line, so
 that deleting the quoted region is not the fix. Texts are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { insertionRegionWithNeighbours, } from '../dist/final/node/index.mjs';

/**
 The text a region covers.

 @param text - passage the region is in

 @param region - offsets read

 @returns The region's text

 @example
 ```ts
 const covered = textOf({ text, region, },);
 ```
 */
function textOf(
  {
    text,
    region,
  }: {
    readonly text: string;
    readonly region: { readonly startOffset: number; readonly endOffset: number; };
  },
): string {
  return text.slice(
    region.startOffset,
    region.endOffset,
  );
}

/**
 Widens the sentence found between two markers.

 @param text - passage holding the sentence

 @param sentence - the inserted sentence, whole

 @returns The widened region and how many neighbours it took

 @example
 ```ts
 const widened = widenedAround({ text: 'A cat. Inserted one. B cat.', sentence: 'Inserted one.', },);
 ```
 */
function widenedAround(
  {
    text,
    sentence,
  }: {
    readonly text: string;
    readonly sentence: string;
  },
): ReturnType<typeof insertionRegionWithNeighbours> {
  /**
   Where the inserted sentence begins.
   */
  const startOffset = text.indexOf(sentence,);
  return insertionRegionWithNeighbours({
    text,
    region: {
      startOffset,
      endOffset: startOffset + sentence.length,
    },
  },);
}

await describe({
  name: insertionRegionWithNeighbours.name,
  children: [
    it({
      name: 'TAKES THE SENTENCE BEFORE AND THE SENTENCE AFTER on the same line, with the separators between them, so '
        + 'a reply that deletes the quoted region deletes the neighbours too',
      fn: async () => {
        /**
         A line of four sentences, the third inserted.
         */
        const text = 'The cat slept. The cat woke. A winter trip was taken. The cat ate.';
        const { region, neighbours, } = widenedAround({
          text,
          sentence: 'A winter trip was taken.',
        },);
        expect(neighbours,).toBe(2,);
        expect(textOf({
          text,
          region,
        },),).toBe('The cat woke. A winter trip was taken. The cat ate.',);
      },
    },),

    it({
      name: 'TAKES ONLY THE SENTENCE BEFORE when the inserted one closes its line, which is where a section ends',
      fn: async () => {
        /**
         The inserted sentence ends the text.
         */
        const text = 'The cat slept. The cat woke. A winter trip was taken.';
        const { region, neighbours, } = widenedAround({
          text,
          sentence: 'A winter trip was taken.',
        },);
        expect(neighbours,).toBe(1,);
        expect(textOf({
          text,
          region,
        },),).toBe('The cat woke. A winter trip was taken.',);
      },
    },),

    it({
      name: 'TAKES ONLY THE SENTENCE AFTER when the inserted one opens its line, and never reaches across a line break',
      fn: async () => {
        /**
         A heading line, then a line of two sentences.
         */
        const text = 'The heading\nA winter trip was taken. The cat ate.\nThe cat slept.';
        const { region, neighbours, } = widenedAround({
          text,
          sentence: 'A winter trip was taken.',
        },);
        expect(neighbours,).toBe(1,);
        expect(textOf({
          text,
          region,
        },),).toBe('A winter trip was taken. The cat ate.',);
      },
    },),

    it({
      name: 'TAKES NOBODY when the inserted sentence is alone on its line, and says so with a count of zero, so the '
        + 'caller can refuse a trial whose quoted region would be the sentence alone',
      fn: async () => {
        /**
         Three lines, the middle one inserted.
         */
        const text = 'The cat slept.\n\nA winter trip was taken.\n\nThe cat ate.';
        const { region, neighbours, } = widenedAround({
          text,
          sentence: 'A winter trip was taken.',
        },);
        expect(neighbours,).toBe(0,);
        expect(textOf({
          text,
          region,
        },),).toBe('A winter trip was taken.',);
      },
    },),
  ],
},);
