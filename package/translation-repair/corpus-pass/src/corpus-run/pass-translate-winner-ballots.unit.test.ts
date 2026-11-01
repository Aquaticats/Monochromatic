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

import { wrapReplacementText, } from '@monochromatic-dev/translation-repair-typography';
import {
  translateWinnerBallotsOf,
  type TranslateWinnerRecord,
} from '../../dist/final/node/index.mjs';

/**
 What the translate document carries at slice 5.
 */
const SHIPPED = 'The cat naps on the sill.';

/**
 What the lane's round chose at slice 44 of a settled artifact, as the judges saw it: one line to a paragraph, a note
 mark inside a quotation. Invented in the shape of XingZ6022's slice 44, whose wording is not copied here.
 */
const JUDGED_QUOTATION = '> Hey, have you seen the new cat video? It is called something like Purr-something Light[^3]. '
  + 'It is really fun. Other videos are all naps, stretches or naps, naps, stretches, but this video is naps, naps, '
  + 'naps, stretches. It is really fun.\n\nI vaguely remember that, back then, her purr was actually full of laughter.';

/**
 The same wording after the lane's semantic wrap, which is what the translate document and the delivery ledger carry:
 a break after every clause, and the quotation's `> ` prefix on each line the wrap added.
 */
const WRAPPED_QUOTATION = '> Hey,\n> have you seen the new cat video?\n> It is called something like Purr-something Light[^3].\n'
  + '> It is really fun.\n> Other videos are all naps,\n> stretches or naps,\n> naps,\n> stretches,\n'
  + '> but this video is naps,\n> naps,\n> naps,\n> stretches.\n> It is really fun.\n\n'
  + 'I vaguely remember that,\nback then,\nher purr was actually full of laughter.';

/**
 A record the lane's round settled, whose round chose position 2 with the given ballots.

 @param sliceIndex - slice the record answers
 @param origin - whether the round chose fresh wording or kept the archive's
 @param text - wording the lane delivers, after the semantic wrap
 @param ballots - ballots that named position 2
 @param judged - wording the judges saw, which the round's result carries beside the delivered one; the delivered
 wording by default
 @param changed - whether the delivered wording differs from the archive's, true by default
 @param disposition - what the driver did with the round's result, `stage-result` by default

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
    judged = text,
    changed = true,
    disposition = 'stage-result',
  }: {
    readonly sliceIndex: number;
    readonly origin: 'fresh' | 'incumbent';
    readonly text: string;
    readonly ballots: number;
    readonly judged?: string;
    readonly changed?: boolean;
    readonly disposition?: TranslateWinnerRecord['disposition'];
  },
): TranslateWinnerRecord {
  // The round's result carries the wording the judges saw; the count never reads it, and keeping it in the fixture
  // keeps it the shape of a real record, whose result holds more than the part this reads.
  const stageResult = {
    text: judged,
    origin,
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
  };
  return {
    sliceIndex,
    outputText: text,
    changed,
    disposition,
    stageResult,
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
      name: 'COUNTS THE BALLOTS OF A WORDING THE SEMANTIC WRAP BROKE ACROSS LINES, a quotation and its second '
        + 'paragraph included: the ledger row holds the delivered form, the judges saw the one-line form, and two '
        + 'ballots named it (XingZ6022 slice 44, XingZ6023 slices 13, 44, 68 and 69)',
      fn: async () => {
        const counted = translateWinnerBallotsOf({
          records: [
            recordOf({
              sliceIndex: 44,
              origin: 'fresh',
              text: WRAPPED_QUOTATION,
              judged: JUDGED_QUOTATION,
              ballots: 2,
            },),
          ],
          comparison: [
            {
              sliceIndex: 44,
              translateText: WRAPPED_QUOTATION,
            },
          ],
        },);
        // THE DELIVERED FORM IS THE REAL WRAP'S OUTPUT, not a hand-made one the count could agree with by accident.
        expect(wrapReplacementText({ text: JUDGED_QUOTATION, },),).toBe(WRAPPED_QUOTATION,);
        expect([...counted,],).toEqual([
          [
            44,
            2,
          ],
        ],);
      },
    },),
    it({
      name: 'CREDITS NOTHING WHERE THE GUARD REFUSED THE ROUND\'S CHOICE or the wrap made the wording the archive\'s '
        + 'own bytes, though the ledger row equals the delivered text and the round chose fresh wording',
      fn: async () => {
        const counted = translateWinnerBallotsOf({
          records: [
            recordOf({
              sliceIndex: 5,
              origin: 'fresh',
              text: SHIPPED,
              judged: 'A cat naps on the sill, as chosen.',
              ballots: 2,
              disposition: 'refused-quote-loss',
              changed: false,
            },),
            recordOf({
              sliceIndex: 6,
              origin: 'fresh',
              text: 'A cat dozes.',
              judged: 'A cat   dozes.',
              ballots: 2,
              changed: false,
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
        expect([...counted,],).toEqual([],);
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
