/**
 Tests for reading whether a rebuilt carve is the run's own, row by row.

 EACH WAY A ROW CAN DEPART HAS A CASE, in the order the reading asks them:
 the slice index, the original, the placement (both directions: archive text
 where the run had none, and none where it had some), and the archive span.
 A row count that differs is its own answer, read before any row. The first
 departing position is the one reported, so a case departs at a later
 position with an earlier one matching.

 Rows are built from the slices they describe and then changed one field at a
 time, so each case differs from the matching control by exactly what it
 names.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { carveDivergence, } from '../../dist/final/node/index.mjs';

//region Rebuild rows tests

/**
 A row the run recorded, as the function reads it.
 */
type Row = Parameters<typeof carveDivergence>[0]['rows'][number];

/**
 A slice the rebuild carved, as the function reads it.
 */
type Slice = Parameters<typeof carveDivergence>[0]['slices'][number];

/**
 Builds a slice whose archive holds wording for its original.

 @param sliceIndex - global index the slice holds

 @param original - original-side text

 @param archive - archive-side text

 @returns Slice with content on both sides

 @example
 ```ts
 const slice = contentSlice({ sliceIndex: 0, original: '猫睡了。', archive: 'The cat slept.', },);
 ```
 */
function contentSlice(
  {
    sliceIndex,
    original,
    archive,
  }: {
    readonly sliceIndex: number;
    readonly original: string;
    readonly archive: string;
  },
): Slice {
  return {
    source: {
      sliceIndex,
      startOffset: 0,
      endOffset: original.length,
      nodes: [],
      text: original,
    },
    target: {
      sliceIndex,
      startOffset: 0,
      endOffset: archive.length,
      nodes: [],
      text: archive,
    },
  };
}

/**
 Builds a slice whose archive holds no wording for its original.

 @param sliceIndex - global index the slice holds

 @param original - original-side text

 @returns Slice whose archive side is an insertion anchor

 @example
 ```ts
 const slice = insertionSlice({ sliceIndex: 1, original: '猫醒了。', },);
 ```
 */
function insertionSlice(
  {
    sliceIndex,
    original,
  }: {
    readonly sliceIndex: number;
    readonly original: string;
  },
): Slice {
  return {
    source: {
      sliceIndex,
      startOffset: 0,
      endOffset: original.length,
      nodes: [],
      text: original,
    },
    target: {
      kind: 'insertion',
      sliceIndex,
      startOffset: 0,
      endOffset: 0,
      nodes: [],
      text: '',
    },
  };
}

/**
 The row a run that carved this slice would have recorded.

 @param slice - slice the row describes

 @returns Row matching the slice in every field the reading compares

 @example
 ```ts
 const row = rowFor({ slice, },);
 ```
 */
function rowFor({ slice, }: { readonly slice: Slice; },): Row {
  return {
    sliceIndex: slice.target.sliceIndex,
    sourceText: slice.source.text,
    incumbentKind: (slice.target.kind === 'insertion') ? 'absent' : 'present',
    incumbentText: slice.target.text,
    outcome: { kind: 'not-evaluated', },
    shippedText: slice.target.text,
    delivery: { kind: 'incumbent-retained', },
  };
}

/**
 Slice every case carves first, with archive wording.
 */
const SLEPT = contentSlice({
  sliceIndex: 0,
  original: '猫睡了。',
  archive: 'The cat slept.',
},);

/**
 Slice every case carves second, with archive wording.
 */
const WOKE = contentSlice({
  sliceIndex: 1,
  original: '猫醒了。',
  archive: 'The cat woke.',
},);

/**
 A slice whose archive has no wording.
 */
const PURRED = insertionSlice({
  sliceIndex: 1,
  original: '猫呼噜。',
},);

await describe({
  name: carveDivergence.name,
  children: [
    it({
      name: 'READS NO DEPARTURE where every row matches its slice, the control every other case departs from',
      fn: async () => {
        expect(carveDivergence({
          rows: [
            rowFor({ slice: SLEPT, },),
            rowFor({ slice: PURRED, },),
          ],
          slices: [
            SLEPT,
            PURRED,
          ],
        },),).toBe('',);
      },
    },),

    it({
      name: 'NAMES A ROW COUNT that differs before reading any row',
      fn: async () => {
        expect(carveDivergence({
          rows: [rowFor({ slice: SLEPT, },),],
          slices: [
            SLEPT,
            WOKE,
          ],
        },),).toBe('2 slices rebuilt where the run recorded 1',);
      },
    },),

    it({
      name: 'NAMES A SLICE INDEX that moved, reporting the first departing position',
      fn: async () => {
        expect(carveDivergence({
          rows: [
            rowFor({ slice: SLEPT, },),
            {
              ...rowFor({ slice: WOKE, },),
              sliceIndex: 5,
            },
          ],
          slices: [
            SLEPT,
            WOKE,
          ],
        },),).toBe('slice at position 1 has index 1 where the run recorded 5',);
      },
    },),

    it({
      name: 'NAMES ANOTHER ORIGINAL at a position whose index still matches',
      fn: async () => {
        expect(carveDivergence({
          rows: [
            {
              ...rowFor({ slice: SLEPT, },),
              sourceText: '狗睡了。',
            },
            rowFor({ slice: WOKE, },),
          ],
          slices: [
            SLEPT,
            WOKE,
          ],
        },),).toBe('slice at position 0 has another original',);
      },
    },),

    it({
      name: 'NAMES ARCHIVE TEXT WHERE THE RUN HAD NONE: the rebuild found wording at a slice the run recorded '
        + 'as absent',
      fn: async () => {
        expect(carveDivergence({
          rows: [
            {
              ...rowFor({ slice: SLEPT, },),
              incumbentKind: 'absent',
            },
          ],
          slices: [SLEPT,],
        },),).toBe('slice at position 0 has archive text where the run had none',);
      },
    },),

    it({
      name: 'NAMES NO ARCHIVE TEXT WHERE THE RUN HAD SOME: the rebuild carved an insertion at a slice the run '
        + 'recorded wording for',
      fn: async () => {
        expect(carveDivergence({
          rows: [
            {
              ...rowFor({ slice: PURRED, },),
              incumbentKind: 'present',
            },
          ],
          slices: [PURRED,],
        },),).toBe('slice at position 0 has no archive text where the run had some',);
      },
    },),

    it({
      name: 'NAMES ANOTHER ARCHIVE SPAN where the placement matches and the wording does not',
      fn: async () => {
        expect(carveDivergence({
          rows: [
            {
              ...rowFor({ slice: SLEPT, },),
              incumbentText: 'The cat napped.',
            },
          ],
          slices: [SLEPT,],
        },),).toBe('slice at position 0 has another archive span',);
      },
    },),
  ],
},);

//endregion Rebuild rows tests
