/**
 Tests for the pass that keeps repeat groups apart: of two repeats whose
 occurrences share a character of the page, the one covering more text stays,
 a tie goes to the one listed first, and a dropped repeat is dropped whole.
 Fixtures are cat-themed invention only.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { disjointRepeatsOf, } from '../dist/final/node/index.mjs';

/**
 Places one repeat stands, as start and end offsets.
 */
type Places = readonly (readonly [number, number])[];

/**
 Builds a found repeat standing at the given places.

 @param kind - kind of repeat
 @param places - start and end offset of each occurrence, in page order
 @param widths - wording characters of each occurrence, in the same order, where that is not the width between its offsets

 @returns Found repeat whose slot is the whole occurrence

 @example
 ```ts
 const repeat = repeatAt({ kind: 'maximal', places: [[0, 12,], [20, 32,],], },);
 ```
 */
function repeatAt(
  {
    kind,
    places,
    widths = [],
  }: {
    readonly kind: 'maximal' | 'refrain';
    readonly places: Places;
    readonly widths?: readonly number[];
  },
) {
  return {
    kind,
    prefix: (kind === 'maximal') ? '' : '猫咪窗',
    suffix: (kind === 'maximal') ? '' : '台阳盹',
    length: 0,
    spans: places.map(([start, end,], index,) => ({
      startOffset: start,
      endOffset: end,
      quoted: false,
      width: widths[index] ?? (end - start),
      slotStartOffset: start,
      slotEndOffset: end,
    }),),
  };
}

/**
 Builds a maximal repeat standing at the given places.

 @param places - start and end offset of each occurrence, in page order
 @param widths - wording characters of each occurrence, where that is not the width between its offsets

 @returns Maximal found repeat

 @example
 ```ts
 const repeat = maximalAt({ places: [[0, 12,], [20, 32,],], },);
 ```
 */
function maximalAt(
  {
    places,
    widths = [],
  }: {
    readonly places: Places;
    readonly widths?: readonly number[];
  },
) {
  return repeatAt({
    kind: 'maximal',
    places,
    widths,
  },);
}

/**
 Message the refusal of a repeat whose own occurrences overlap carries.
 */
const OWN_OVERLAP_REFUSAL = 'Error: unreachable: two occurrences of one repeat overlap, '
  + 'though both finders keep the occurrences of a repeat apart';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: disjointRepeatsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'Returns nothing for no repeat',
          fn: async () => {
            expect(disjointRepeatsOf({ repeats: [], },),).toEqual([],);
          },
        },),

        it({
          name: 'Keeps every repeat whose occurrences are apart, the one covering most text first',
          fn: async () => {
            /** Covers 24 characters. */
            const small = maximalAt({ places: [[0, 12,], [20, 32,],], },);
            /** Covers 36 characters. */
            const large = maximalAt({ places: [[40, 52,], [60, 72,], [80, 92,],], },);

            expect(disjointRepeatsOf({ repeats: [small, large,], },),).toEqual([large, small,],);
          },
        },),

        it({
          name: 'Drops the repeat covering less text when one occurrence of each overlaps',
          fn: async () => {
            /** Covers 24 characters. */
            const small = maximalAt({ places: [[0, 12,], [30, 42,],], },);
            /** Covers 28 characters, and overlaps the small repeat's first occurrence. */
            const large = maximalAt({ places: [[5, 19,], [50, 64,],], },);

            expect(disjointRepeatsOf({ repeats: [small, large,], },),).toEqual([large,],);
            expect(disjointRepeatsOf({ repeats: [large, small,], },),).toEqual([large,],);
          },
        },),

        it({
          name: 'Keeps the repeat covering more text though it has fewer occurrences and starts later',
          fn: async () => {
            /** Three occurrences of twelve, 36 characters, starting first. */
            const many = maximalAt({ places: [[0, 12,], [30, 42,], [60, 72,],], },);
            /** Two occurrences of 20, 40 characters, each overlapping one of the many. */
            const long = maximalAt({ places: [[28, 48,], [58, 78,],], },);

            expect(disjointRepeatsOf({ repeats: [many, long,], },),).toEqual([long,],);
          },
        },),

        it({
          name: 'Counts the text a repeat covers in wording characters and not in the width between its offsets',
          fn: async () => {
            /** Two places of 24 offsets that hold twelve characters each, so 24 characters over 48 offsets. */
            const wide = maximalAt({ places: [[0, 24,], [100, 124,],], widths: [12, 12,], },);
            /** Two places of 15 offsets that hold fifteen characters each, so 30 characters over 30 offsets. */
            const long = maximalAt({ places: [[10, 25,], [110, 125,],], },);

            expect(disjointRepeatsOf({ repeats: [wide, long,], },),).toEqual([long,],);
          },
        },),

        it({
          name: 'Drops a repeat whole when only one of its occurrences overlaps a kept one',
          fn: async () => {
            /** Covers 40 characters. */
            const kept = maximalAt({ places: [[0, 20,], [100, 120,],], },);
            /** Covers 30 characters; its first occurrence is clear of the kept repeat and its second is not. */
            const dropped = maximalAt({ places: [[40, 55,], [110, 125,],], },);

            expect(disjointRepeatsOf({ repeats: [kept, dropped,], },),).toEqual([kept,],);
          },
        },),

        it({
          name: 'Drops a repeat that stands wholly inside an occurrence of a longer one',
          fn: async () => {
            /** Two occurrences of 30. */
            const outer = maximalAt({ places: [[0, 30,], [100, 130,],], },);
            /** Two occurrences of 12, each inside an occurrence of the outer repeat. */
            const inner = maximalAt({ places: [[10, 22,], [110, 122,],], },);

            expect(disjointRepeatsOf({ repeats: [inner, outer,], },),).toEqual([outer,],);
          },
        },),

        it({
          name: 'Keeps repeats whose occurrences only touch a kept occurrence, at its start and at its end',
          fn: async () => {
            /** Covers 36 characters, so it is kept first; one occurrence is [12, 24). */
            const middle = maximalAt({ places: [[12, 24,], [100, 112,], [200, 212,],], },);
            /** Covers 24 characters; one occurrence ends at 12 where the kept one begins. */
            const before = maximalAt({ places: [[0, 12,], [300, 312,],], },);
            /** Covers 24 characters; one occurrence begins at 24 where the kept one ends. */
            const after = maximalAt({ places: [[24, 36,], [400, 412,],], },);

            expect(disjointRepeatsOf({ repeats: [after, before, middle,], },),).toEqual([middle, before, after,],);
          },
        },),

        it({
          name: 'Lets a dropped repeat block nothing, so a repeat that overlaps only it stays',
          fn: async () => {
            /** Covers 60 characters, the most. */
            const first = maximalAt({ places: [[0, 15,], [100, 115,], [200, 215,], [300, 315,],], },);
            /** Covers 40 characters and overlaps the first repeat's first occurrence. */
            const middle = maximalAt({ places: [[10, 30,], [400, 420,],], },);
            /** Covers 24 characters and overlaps the middle repeat's second occurrence only. */
            const last = maximalAt({ places: [[405, 417,], [500, 512,],], },);

            expect(disjointRepeatsOf({ repeats: [middle, last, first,], },),).toEqual([first, last,],);
          },
        },),

        it({
          name: 'Decides a tie by the order the groups are listed in, whatever order they are given in',
          fn: async () => {
            /** Covers 24 characters from offset 40. */
            const later = maximalAt({ places: [[40, 52,], [90, 102,],], },);
            /** Covers 24 characters from offset 0, and overlaps the later repeat's second occurrence. */
            const earlier = maximalAt({ places: [[0, 12,], [95, 107,],], },);

            expect(disjointRepeatsOf({ repeats: [later, earlier,], },),).toEqual([earlier,],);
            expect(disjointRepeatsOf({ repeats: [earlier, later,], },),).toEqual([earlier,],);
          },
        },),

        it({
          name: 'Decides a tie between a maximal repeat and a refrain that start together for the maximal repeat',
          fn: async () => {
            /** Covers 46 characters. */
            const maximal = maximalAt({ places: [[0, 23,], [25, 48,],], },);
            /** Covers 46 characters and overlaps the maximal repeat at both places. */
            const refrain = repeatAt({ kind: 'refrain', places: [[0, 23,], [25, 48,],], },);

            expect(disjointRepeatsOf({ repeats: [refrain, maximal,], },),).toEqual([maximal,],);
          },
        },),

        it({
          name: 'Keeps a refrain that covers more text than a maximal repeat it overlaps',
          fn: async () => {
            /** Covers 46 characters. */
            const maximal = maximalAt({ places: [[0, 23,], [25, 48,],], },);
            /** Covers 55 characters: five clauses of eleven. */
            const refrain = repeatAt({
              kind: 'refrain',
              places: [[0, 11,], [12, 23,], [25, 36,], [37, 48,], [60, 71,],],
            },);

            expect(disjointRepeatsOf({ repeats: [maximal, refrain,], },),).toEqual([refrain,],);
          },
        },),

        it({
          name: 'Finds the overlap among many kept occurrences wherever it falls',
          fn: async () => {
            /** Forty occurrences of ten, 400 characters, with a gap of ten after each. */
            const wall = maximalAt({
              places: Array.from({ length: 40, }, (_, index,) => [index * 20, (index * 20) + 10,] as const,),
            },);
            /** Fills two gaps and the stretch after the wall, touching the wall at each end. */
            const gaps = maximalAt({ places: [[10, 20,], [490, 500,], [790, 800,],], },);
            /** Fills a gap, then reaches into the wall's eleventh occurrence. */
            const hit = maximalAt({ places: [[30, 40,], [205, 215,],], },);

            expect(disjointRepeatsOf({ repeats: [hit, gaps, wall,], },),).toEqual([wall, gaps,],);
          },
        },),

        it({
          name: 'Leaves the list it is given in its order',
          fn: async () => {
            /** Covers 24 characters. */
            const small = maximalAt({ places: [[0, 12,], [20, 32,],], },);
            /** Covers 36 characters. */
            const large = maximalAt({ places: [[40, 52,], [60, 72,], [80, 92,],], },);
            /** The list handed in. */
            const given = [small, large,];
            disjointRepeatsOf({ repeats: given, },);

            expect(given,).toEqual([small, large,],);
          },
        },),

        it({
          name: 'Refuses a repeat whose own occurrences overlap',
          fn: async () => {
            /** Two occurrences sharing characters 6 to 12. */
            const own = maximalAt({ places: [[0, 12,], [6, 18,],], },);
            /** What the pass throws. */
            const refusal = caught(function keepOverlapping(): unknown {
              return disjointRepeatsOf({ repeats: [own,], },);
            },);

            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe(OWN_OVERLAP_REFUSAL,);
          },
        },),
      ],
    },),
  ],
},);
