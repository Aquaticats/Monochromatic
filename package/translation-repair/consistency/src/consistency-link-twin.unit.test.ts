/**
 Tests for the text a reader sees of an English with links in it (`consistency-link-twin.ts`, n169): the twin with each link that
 shows nothing taken out and each link of closing marks written as one closing bracket, and the offsets that carry an answer about
 the twin back to the English.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  foldedLinksOf,
  foldWithOffsets,
  linkTwinOf,
  NO_FOLDED_LINKS,
  originalOffsetsOf,
} from '../dist/final/node/index.mjs';

/**
 The twin of an English, its links read by the parse.

 @param english - English, which folds unit for unit

 @param closing - offsets of the units that close an emphasis

 @returns Twin

 @example
 ```ts
 const twin = twinOf({ english: 'kit[](u)\'kit', closing: new Set(), },); // twin.folded is "kit'kit"
 ```
 */
function twinOf(
  {
    english,
    closing,
  }: {
    readonly english: string;
    readonly closing: ReadonlySet<number>;
  },
): ReturnType<typeof linkTwinOf> {
  /**
   The English in the fold.
   */
  const folded = foldWithOffsets({ text: english, },);
  return linkTwinOf({
    folded: folded.folded,
    originals: folded.originals,
    closing,
    links: foldedLinksOf({
      english,
      folded,
    },),
  },);
}

await describe({
  name: 'the link twin',
  concurrency: 1,
  children: [
    describe({
      name: linkTwinOf.name,
      concurrency: 1,
      children: [
        it({
          name: 'TAKES OUT A LINK THAT SHOWS NOTHING, with the offset of every unit that stays and the start of the link taken out before it',
          fn: async () => {
            /**
             The twin of an English with one empty link.
             */
            const twin = twinOf({ english: 'kit[](u)\'kit', closing: new Set(), },);
            expect(twin.folded,).toBe('kit\'kit',);
            expect(twin.starts,).toEqual([0, 1, 2, 8, 9, 10, 11, 12,],);
            expect([...twin.removedBefore.entries(),],).toEqual([[3, [3,],],],);
          },
        },),

        it({
          name: 'TAKES OUT EVERY LINK OF A RUN, an image and a reference among them, and notes the start of each before the next unit',
          fn: async () => {
            /**
             The twin of an English with a run of empty links.
             */
            const twin = twinOf({ english: 'a[](u)![](v)[]()\'b', closing: new Set(), },);
            expect(twin.folded,).toBe('a\'b',);
            expect(twin.starts,).toEqual([0, 16, 17, 18,],);
            expect([...twin.removedBefore.entries(),],).toEqual([[1, [1, 6, 12,],],],);
          },
        },),

        it({
          name: 'TAKES OUT A LINK AT THE START OR THE END OF THE TEXT, noting it before the end of the twin when nothing follows',
          fn: async () => {
            /**
             The twins of English texts with an empty link at each edge.
             */
            const twins = [
              twinOf({ english: '[](u)kit', closing: new Set(), },),
              twinOf({ english: 'kit[](u)', closing: new Set(), },),
            ];
            expect(twins.map(function foldedOf({ folded, },): string {
              return folded;
            },),).toEqual(['kit', 'kit',],);
            expect(twins.map(function startsOf({ starts, },): readonly number[] {
              return starts;
            },),).toEqual([[5, 6, 7, 8,], [0, 1, 2, 8,],],);
            expect(twins.map(function removedOf({ removedBefore, },): readonly (readonly [number, readonly number[]])[] {
              return [...removedBefore.entries(),];
            },),).toEqual([[[0, [0,],],], [[3, [3,],],],],);
          },
        },),

        it({
          name: 'WRITES A LINK OF CLOSING MARKS AS ONE CLOSING BRACKET, whatever marks the label shows and however its destination is written',
          fn: async () => {
            /**
             The twins of English texts with a link of closing marks.
             */
            const twins = [
              twinOf({ english: String.raw`kit[\]](u)'kit`, closing: new Set(), },),
              twinOf({ english: 'kit[\u{201D}\u{201D}](u "t")\'kit', closing: new Set(), },),
            ];
            expect(twins.map(function foldedOf({ folded, },): string {
              return folded;
            },),).toEqual(['kit]\'kit', 'kit]\'kit',],);
            expect(twins.map(function originalsOf({ originals, },): readonly string[] {
              return originals.slice(
                3,
                5,
              );
            },),).toEqual([[']', '\'',], [']', '\'',],],);
            expect(twins[0]?.starts,).toEqual([0, 1, 2, 3, 10, 11, 12, 13, 14,],);
          },
        },),

        it({
          name: 'CARRIES THE CLOSING DELIMITERS OF EMPHASIS to the offsets of the twin, before and after a link taken out',
          fn: async () => {
            expect(twinOf({ english: '*kit*[](u)\'s', closing: new Set([4,],), },).closing,).toEqual(new Set([4,],),);
            expect(twinOf({ english: '[](u)*kit*', closing: new Set([9,],), },).closing,).toEqual(new Set([4,],),);
            expect(twinOf({ english: '[](u)*kit*', closing: new Set(), },).closing,).toEqual(new Set(),);
          },
        },),

        it({
          name: 'LEAVES THE TEXT AS IT IS where the parse holds no link that shows nothing or closing marks',
          fn: async () => {
            /**
             The twin of an English with a link that shows a word.
             */
            const twin = twinOf({ english: 'kit[x](u)\'kit', closing: new Set(), },);
            expect(twin.folded,).toBe('kit[x](u)\'kit',);
            expect(twin.starts,).toEqual([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,],);
            expect(twin.removedBefore.size,).toBe(0,);
            expect(NO_FOLDED_LINKS.emptyEnds.size,).toBe(0,);
          },
        },),

        it({
          name: 'TAKES OUT A THOUSAND LINKS IN ONE PASS',
          fn: async () => {
            /**
             The twin of an English with a thousand empty links before a word.
             */
            const twin = twinOf({ english: `kit${'[](u)'.repeat(1_000,)}'kit`, closing: new Set(), },);
            expect(twin.folded,).toBe('kit\'kit',);
            expect(twin.removedBefore.get(3,)?.length,).toBe(1_000,);
          },
        },),
      ],
    },),

    describe({
      name: originalOffsetsOf.name,
      concurrency: 1,
      children: [
        it({
          name: 'NAMES THE OFFSET OF EACH UNIT OF THE TWIN AND THE START OF EACH LINK TAKEN OUT DIRECTLY BEFORE IT',
          fn: async () => {
            /**
             The twin of an English with an empty link.
             */
            const twin = twinOf({ english: 'kit[](u)\'kit', closing: new Set(), },);
            expect(originalOffsetsOf({ twin, found: new Set([3,],), },),).toEqual(new Set([8, 3,],),);
            expect(originalOffsetsOf({ twin, found: new Set([1,],), },),).toEqual(new Set([1,],),);
            expect(originalOffsetsOf({ twin, found: new Set([99,],), },),).toEqual(new Set(),);
            expect(originalOffsetsOf({ twin, found: new Set(), },),).toEqual(new Set(),);
          },
        },),
      ],
    },),
  ],
},);
