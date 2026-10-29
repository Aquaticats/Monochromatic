/**
 Tests for the counts across the entries of the displacement probe, and for
 which entry carries anything worth printing.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  corpusTotals,
  isNotable,
} from '../../dist/final/node/index.mjs';
import { rowOf, } from './displacement-probe-rows.test-fixture.ts';

/**
 One relocation candidate, as the classification gives it.
 */
const MOVED = {
  high: 5,
  low: 4,
  surplus: 472,
  deficit: 472,
} as const;

await describe({
  name: 'displacement-probe-totals',
  children: [
    describe({
      name: corpusTotals.name,
      children: [
        it({
          name: 'COUNTS nothing for no entries',
          fn: async () => {
            expect(corpusTotals({ rows: [], },),).toEqual({
              slices: 0,
              fellBack: 0,
              relocationCandidates: 0,
              untranslated: 0,
              targetOnly: 0,
              transcriptionSuspects: 0,
              otherImbalances: 0,
            },);
          },
        },),
        it({
          name: 'COUNTS one entry that read its own baseline, with one in each class, as one each',
          fn: async () => {
            expect(corpusTotals({
              rows: [rowOf({ changes: {
                sliceCount: 9,
                relocationCandidates: [MOVED,],
                transcriptionSuspects: [5,],
                untranslated: [1,],
                targetOnly: [2,],
                otherImbalances: [8,],
              }, },),],
            },),).toEqual({
              slices: 9,
              fellBack: 0,
              relocationCandidates: 1,
              untranslated: 1,
              targetOnly: 1,
              transcriptionSuspects: 1,
              otherImbalances: 1,
            },);
          },
        },),
        it({
          name: 'SUMS several entries, counting an entry that fell back to the corpus baseline and every class by its length',
          fn: async () => {
            expect(corpusTotals({
              rows: [
                rowOf({ changes: {
                  sliceCount: 3,
                  baselineFrom: 'corpus-reference',
                  untranslated: [0, 1,],
                }, },),
                rowOf({ changes: {
                  entryId: 'Tabby',
                  sliceCount: 4,
                  relocationCandidates: [MOVED, MOVED,],
                  targetOnly: [2, 3, 4,],
                  otherImbalances: [6,],
                }, },),
              ],
            },),).toEqual({
              slices: 7,
              fellBack: 1,
              relocationCandidates: 2,
              untranslated: 2,
              targetOnly: 3,
              transcriptionSuspects: 0,
              otherImbalances: 1,
            },);
          },
        },),
      ],
    },),
    describe({
      name: isNotable.name,
      children: [
        it({
          name: 'PASSES OVER an entry no class named, even one that fell back to the corpus baseline',
          fn: async () => {
            expect(isNotable({ row: rowOf({ changes: { baselineFrom: 'corpus-reference', }, },), },),).toBe(false,);
          },
        },),
        it({
          name: 'NOTES an entry a relocation candidate named',
          fn: async () => {
            expect(isNotable({ row: rowOf({ changes: { relocationCandidates: [MOVED,], }, },), },),).toBe(true,);
          },
        },),
        it({
          name: 'NOTES an entry an untranslated slice named',
          fn: async () => {
            expect(isNotable({ row: rowOf({ changes: { untranslated: [2,], }, },), },),).toBe(true,);
          },
        },),
        it({
          name: 'NOTES an entry a target-only slice named',
          fn: async () => {
            expect(isNotable({ row: rowOf({ changes: { targetOnly: [2,], }, },), },),).toBe(true,);
          },
        },),
        it({
          name: 'NOTES an entry only a one-ended surplus named',
          fn: async () => {
            expect(isNotable({ row: rowOf({ changes: { otherImbalances: [2,], }, },), },),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
