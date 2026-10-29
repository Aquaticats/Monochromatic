/**
 Tests for the bench draw and the width sweep it runs.

 Both decide what a width comparison MEASURES, and both fail silently: a draw
 that varied between runs would compare widths over different samples and
 still print a clean table, and a sweep that stopped at the old roster length
 would quietly stop measuring the widest case the day a model is added.

 Fixtures are invented. No corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BenchDrawError,
  BenchReportError,
  benchWidths,
  type DrawableSlice,
  orderBySourceSize,
  pickSpreadSample,
} from '../../dist/final/node/index.mjs';

/**
 Builds one drawable slice of a given size.

 @param entryId - entry it belongs to

 @param index - position within that entry

 @param size - source characters

 @returns Slice the draw can order

 @example
 ```ts
 const slice = sized({ entryId: 'Mittens', index: 0, size: 40, },);
 ```
 */
function sized(
  {
    entryId,
    index,
    size,
  }: {
    readonly entryId: string;
    readonly index: number;
    readonly size: number;
  },
): DrawableSlice {
  return {
    entryId,
    index,
    sourceText: '猫'.repeat(size,),
  };
}

/**
 Twenty slices spanning one to twenty characters, in scrambled order so an
 accidental dependence on input order shows up.
 */
const SPREAD: readonly DrawableSlice[] = Array.from(
  { length: 20, },
  function toSlice(
    _unused,
    position,
  ): DrawableSlice {
    return sized({
      entryId: `entry-${String(position % 3,)}`,
      index: position,
      // Interleaved rather than ascending: consecutive positions differ by ten
      // characters, so the input arrives nowhere near sorted.
      size: ((position * 7) % 20) + 1,
    },);
  },
);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: orderBySourceSize.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'orders by source size regardless of the order slices arrive in, '
            + 'which is what makes the draw a function of the corpus rather than of '
            + 'the directory listing',
          fn: async () => {
            const sizes = orderBySourceSize({ slices: SPREAD, },)
              .map(function toSize(slice,): number {
                return slice.sourceText
                  .length;
              },);
            expect(sizes,).toEqual([...sizes,].toSorted(function ascending(
              left,
              right,
            ): number {
              return left - right;
            },),);
          },
        },),

        it({
          name: 'breaks size ties by entry then position, so two slices of equal '
            + 'size never swap places between runs. Stable sorting alone would not '
            + 'give this, since the input order is the corpus listing',
          fn: async () => {
            /**
             Three same-size slices, presented worst-first.
             */
            const tied: readonly DrawableSlice[] = [
              sized({
                entryId: 'Whiskers',
                index: 2,
                size: 5,
              },),
              sized({
                entryId: 'Mittens',
                index: 9,
                size: 5,
              },),
              sized({
                entryId: 'Whiskers',
                index: 1,
                size: 5,
              },),
            ];
            expect(orderBySourceSize({ slices: tied, },)
              .map(function toName(slice,): string {
                return `${slice.entryId}#${String(slice.index,)}`;
              },),)
              .toEqual([
                'Mittens#9',
                'Whiskers#1',
                'Whiskers#2',
              ],);
          },
        },),

        it({
          name: 'BREAKS AN ENTRY TIE BY CODE POINT, the same on every machine, so a capitalised entry '
            + 'name comes before a lower-case one and a name past U+FFFF after one inside it: the '
            + 'runtime locale ordered them by its own collation, so two machines could draw different '
            + 'benches from one corpus (ledger B95)',
          fn: async () => {
            /**
             Same-size slices whose entry names a locale collation and code
             points order differently.
             */
            const tied: readonly DrawableSlice[] = [
              sized({
                entryId: 'mooncat',
                index: 0,
                size: 5,
              },),
              sized({
                entryId: 'Tabby',
                index: 0,
                size: 5,
              },),
              sized({
                entryId: 'cat\u{1F431}',
                index: 0,
                size: 5,
              },),
              sized({
                entryId: 'cat\u{FF5E}',
                index: 0,
                size: 5,
              },),
            ];
            expect(orderBySourceSize({ slices: tied, },)
              .map(function toEntry(slice,): string {
                return slice.entryId;
              },),)
              .toEqual([
                'Tabby',
                'cat\u{FF5E}',
                'cat\u{1F431}',
                'mooncat',
              ],);
          },
        },),
      ],
    },),

    describe({
      name: pickSpreadSample.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SKIPS the extremes, which is the whole reason the draw takes '
            + 'stratum midpoints: the smallest slice in this corpus is a 3-character '
            + 'source against a 226-character translation, and a bench starting '
            + 'there measures the aligner rather than the judges',
          fn: async () => {
            const drawn = pickSpreadSample({
              slices: SPREAD,
              count: 4,
            },)
              .map(function toSize(slice,): number {
                return slice.sourceText
                  .length;
              },);
            expect(drawn,).toEqual([
              3,
              8,
              13,
              18,
            ],);
          },
        },),

        it({
          name: 'draws the same sample every time, since widths compared over '
            + 'different samples cannot be compared at all',
          fn: async () => {
            expect(pickSpreadSample({
              slices: SPREAD,
              count: 6,
            },),).toEqual(pickSpreadSample({
              slices: [...SPREAD,].toReversed(),
              count: 6,
            },),);
          },
        },),

        it({
          name: 'returns every slice, in size order and without repeats, when more '
            + 'are asked for than exist',
          fn: async () => {
            const drawn = pickSpreadSample({
              slices: SPREAD,
              count: 500,
            },);
            expect(drawn,).toHaveLength(SPREAD.length,);
            expect(new Set(drawn.map(function toIndex(slice,): number {
              return slice.index;
            },),).size,).toBe(SPREAD.length,);
          },
        },),

        it({
          name: 'refuses an empty pool rather than returning an empty bench, which '
            + 'would report every width as agreeing perfectly',
          fn: async () => {
            /**
             What drawFromNothing raised, read for its class as well as its wording.
             */
            const refusalOfDrawFromNothing = caught(function drawFromNothing(): void {
              pickSpreadSample({
                slices: [],
                count: 3,
              },);
            },);

            expect(refusalOfDrawFromNothing,).toBeInstanceOf(BenchDrawError,);
            expect(String(refusalOfDrawFromNothing,),).toBe('BenchDrawError: a bench sample cannot be drawn from no slices',);
          },
        },),
      ],
    },),

    describe({
      name: benchWidths.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'sweeps every width from two up to the WHOLE roster, derived from '
            + 'its length: a bench that hardcoded six would stop measuring the '
            + 'widest case the day the provider adds a model',
          fn: async () => {
            expect(benchWidths({ roster: [
              'a',
              'b',
              'c',
              'd',
              'e',
              'f',
            ], },).widths,)
              .toEqual([
                2,
                3,
                4,
                5,
                6,
              ],);
          },
        },),

        it({
          name: 'repeats a MIDDLE width rather than an extreme, since that repeat '
            + 'is the run-to-run band every width difference has to clear and the '
            + 'ends are its least representative points',
          fn: async () => {
            expect(benchWidths({ roster: [
              'a',
              'b',
              'c',
              'd',
              'e',
              'f',
            ], },),).toEqual({
              widths: [
                2,
                3,
                4,
                5,
                6,
              ],
              repeated: 4,
            },);
          },
        },),

        it({
          name: 'refuses a roster with nothing to vary, rather than benching one '
            + 'width and reporting a comparison',
          fn: async () => {
            /**
             What benchOneModel raised, read for its class as well as its wording.
             */
            const refusalOfBenchOneModel = caught(function benchOneModel(): void {
              benchWidths({ roster: ['a',], },);
            },);

            expect(refusalOfBenchOneModel,).toBeInstanceOf(BenchReportError,);
            expect(String(refusalOfBenchOneModel,),).toBe('BenchReportError: a roster of 1 cannot be benched: nothing to vary',);
          },
        },),
      ],
    },),
  ],
},);
