/**
 Tests for the count of judges who chose the translate lane's text in the lane's own round, which the consolidation
 reads where the final round cannot seat two judges (owner, 2026-10-29, "Ship the two-ballot text").

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  translateWinnerBallotsOf,
  type TranslateWinnerRecord,
} from '../../dist/final/node/index.mjs';

/**
 What the translate document carries at slice 5.
 */
const SHIPPED = 'The cat naps on the sill.';

/**
 A record the lane's round settled, whose round chose position 2 with the given ballots.

 @param sliceIndex - slice the record answers
 @param origin - whether the round chose fresh wording or kept the archive's
 @param text - wording the round ships
 @param ballots - ballots that named position 2

 @returns Record shaped as the part of the lane's record this reads

 @example
 ```ts
 const record = recordOf({ sliceIndex: 5, origin: 'fresh', text: SHIPPED, ballots: 2, },);
 ```
 */
function recordOf(
  {
    sliceIndex,
    origin,
    text,
    ballots,
  }: {
    readonly sliceIndex: number;
    readonly origin: 'fresh' | 'incumbent';
    readonly text: string;
    readonly ballots: number;
  },
): TranslateWinnerRecord {
  return {
    sliceIndex,
    stageResult: {
      origin,
      text,
      shippedIndex: 2,
      perCandidate: [
        {
          index: 1,
          ballots: 0,
          fullVotes: 0,
          selfVotes: 0,
          weight: 0,
        },
        {
          index: 2,
          ballots,
          fullVotes: ballots,
          selfVotes: 0,
          weight: ballots,
        },
      ],
    },
  };
}

await describe({
  name: translateWinnerBallotsOf.name,
  children: [
    it({
      name: 'COUNTS THE BALLOTS THAT NAMED THE WORDING THE DOCUMENT CARRIES, by slice',
      fn: async () => {
        const counted = translateWinnerBallotsOf({
          records: [
            recordOf({
              sliceIndex: 5,
              origin: 'fresh',
              text: SHIPPED,
              ballots: 2,
            },),
            recordOf({
              sliceIndex: 6,
              origin: 'fresh',
              text: 'A cat dozes.',
              ballots: 3,
            },),
          ],
          comparison: [
            {
              sliceIndex: 5,
              translateText: SHIPPED,
            },
            {
              sliceIndex: 6,
              translateText: 'A cat dozes.',
            },
          ],
        },);
        expect([...counted,],).toEqual([
          [
            5,
            2,
          ],
          [
            6,
            3,
          ],
        ],);
      },
    },),
    it({
      name: 'CREDITS NOTHING WHERE THE ROUND KEPT THE ARCHIVE, where assembly trimmed the wording the judges saw, '
        + 'where no ballot named the shipped position, or where the ledger has no row',
      fn: async () => {
        const counted = translateWinnerBallotsOf({
          records: [
            recordOf({
              sliceIndex: 5,
              origin: 'incumbent',
              text: SHIPPED,
              ballots: 2,
            },),
            recordOf({
              sliceIndex: 6,
              origin: 'fresh',
              text: 'A cat dozes.',
              ballots: 2,
            },),
            recordOf({
              sliceIndex: 7,
              origin: 'fresh',
              text: 'A cat purrs.',
              ballots: 0,
            },),
            recordOf({
              sliceIndex: 8,
              origin: 'fresh',
              text: 'A cat stretches.',
              ballots: 2,
            },),
          ],
          comparison: [
            {
              sliceIndex: 5,
              translateText: SHIPPED,
            },
            {
              sliceIndex: 6,
              translateText: 'A cat dozes, trimmed.',
            },
            {
              sliceIndex: 7,
              translateText: 'A cat purrs.',
            },
          ],
        },);
        expect([...counted,],).toEqual([],);
      },
    },),
  ],
},);
