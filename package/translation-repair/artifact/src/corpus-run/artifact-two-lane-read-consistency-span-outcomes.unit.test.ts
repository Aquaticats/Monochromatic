/**
 Tests for recounting what became of each place of a group the bench decided: the chosen place is chosen, a place with the same
 words is consistent, a place the chosen words are carried to ends as its carry allows, every place is untouched when no place won,
 and the carries recorded are one for each receiving place in order.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ConsistencyRendering, } from '@monochromatic-dev/translation-repair-consistency';
import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  type ArtifactConsistencyBallot,
  type ArtifactConsistencyCarry,
  type ArtifactConsistencyGroup,
  type ArtifactConsistencyOccurrence,
  assertSpanOutcomesFollow,
} from '../../dist/final/node/index.mjs';

/**
 Where the group sits in the section, for the refusals.
 */
const GROUP = 'cat.consistency.groups[0]';

/**
 The group the cases read: two places whose three ballots choose the first.
 */
type AskedGroup = ArtifactConsistencyGroup & { readonly verdict: { readonly kind: 'asked'; }; };

/**
 The renderings of the two places in the counter's form: the first place's words and the second place's other words.
 */
const RENDERINGS: readonly ConsistencyRendering[] = [
  { kind: 'located', quote: 'The cat naps', core: 'The cat naps', start: 0, },
  { kind: 'located', quote: 'the kitten naps', core: 'the kitten naps', start: 6, },
];

/**
 The carry of the second place: the bench chose the chosen words moved in by code.
 */
const CARRY: ArtifactConsistencyCarry = {
  label: 'OCCURRENCE 2',
  candidates: [
    { label: 'CANDIDATE 1', kind: 'current', writers: [], text: 'Later the kitten naps in the sun.', },
    { label: 'CANDIDATE 2', kind: 'splice', writers: [], text: 'Later the cat naps in the sun.', },
  ],
  ballots: [
    { seat: 'hyper-cat', reason: 'It words the original best.', choice: 'CANDIDATE 2', },
    { seat: 'bedrock-cat', reason: 'It words the original best.', choice: 'CANDIDATE 2', },
  ],
  usable: 2,
  quorum: 2,
  decision: { kind: 'chosen', label: 'CANDIDATE 2', },
};

/**
 One ballot, choosing the first place and quoting both.
 */
function ballotOf({ seat, }: { readonly seat: string; },): ArtifactConsistencyBallot {
  return {
    seat,
    reason: 'The first place words it best.',
    quotes: [
      { label: 'OCCURRENCE 1', quote: 'The cat naps.', },
      { label: 'OCCURRENCE 2', quote: 'the kitten naps', },
    ],
    choice: 'OCCURRENCE 1',
    keep: [],
  };
}

/**
 The group with the outcomes of its two places, the carries recorded and the winner named.

 @param outcomes - what became of the first and the second place

 @param carries - carries recorded

 @param winner - how the count ended

 @returns Group

 @example
 ```ts
 const group = groupOf({ outcomes: ['chosen', 'replaced',], carries: [CARRY,], winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },);
 ```
 */
function groupOf(
  {
    outcomes,
    carries,
    winner,
  }: {
    readonly outcomes: readonly [ArtifactConsistencyOccurrence['outcome'], ArtifactConsistencyOccurrence['outcome'],];
    readonly carries: readonly ArtifactConsistencyCarry[];
    readonly winner: AskedGroup['verdict']['winner'];
  },
): AskedGroup {
  return {
    repeat: { kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },
    occurrences: [
      {
        label: 'OCCURRENCE 1',
        sliceIndex: 0,
        startOffset: 6,
        endOffset: 18,
        line: 3,
        rendering: { kind: 'located', quote: 'The cat naps', start: 0, },
        outcome: outcomes[0],
      },
      {
        label: 'OCCURRENCE 2',
        sliceIndex: 1,
        startOffset: 30,
        endOffset: 42,
        line: 5,
        rendering: { kind: 'located', quote: 'the kitten naps', start: 6, },
        outcome: outcomes[1],
      },
    ],
    unplaced: [],
    verdict: {
      kind: 'asked',
      ballots: ['hyper-cat', 'bedrock-cat', 'synthetic-cat',].map(function ballotFor(seat,): ArtifactConsistencyBallot {
        return ballotOf({ seat, },);
      },),
      usable: 3,
      quorum: 2,
      winner,
    },
    carries,
  };
}

/**
 What the recount of the outcomes of a group says: `holds` where it holds, the refusal as text where it does not.

 @param group - group, asked of the bench

 @returns Refusal as text, or `holds`

 @throws {@link Error} when the recount threw something other than an artifact parse refusal

 @example
 ```ts
 const said = recountOf({ group: groupOf({ outcomes: ['chosen', 'replaced',], carries: [CARRY,], winner, },), },);
 ```
 */
function recountOf({ group, }: { readonly group: AskedGroup; },): string {
  try {
    assertSpanOutcomesFollow({
      group,
      renderings: RENDERINGS,
      path: GROUP,
      marks: 'compared',
      partialUnits: true,
    },);
    return 'holds';
  }
  catch (error) {
    if (!(error instanceof ArtifactParseError))
      throw error;
    return String(error,);
  }
}

/**
 The winner of the count when the first place won.
 */
const FIRST_WON = { kind: 'chosen', label: 'OCCURRENCE 1', } as const;

await describe({
  name: '',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    describe({
      name: assertSpanOutcomesFollow.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HOLDS FOR A CHOSEN PLACE, AND A PLACE THE CHOSEN WORDS ARE CARRIED TO THAT ENDS AS ITS CARRY ALLOWS',
          fn: async () => {
            expect(recountOf({
              group: groupOf({ outcomes: ['chosen', 'replaced',], carries: [CARRY,], winner: FIRST_WON, },),
            },),).toBe('holds',);
          },
        },),
        it({
          name: 'REFUSES A CHOSEN PLACE THAT IS NOT CHOSEN, naming the outcome the ballots and the renderings give',
          fn: async () => {
            expect(recountOf({
              group: groupOf({ outcomes: ['untouched', 'replaced',], carries: [CARRY,], winner: FIRST_WON, },),
            },),).toBe(
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[0].outcome: `
                + 'expected chosen, as the ballots and the renderings give for this place.',
            );
          },
        },),
        it({
          name: 'EXPECTS EVERY PLACE UNTOUCHED WHEN NO PLACE WON, whatever the places record',
          fn: async () => {
            /**
             The count that ended with no winner.
             */
            const none = { kind: 'none', reason: 'no-majority', } as const;
            expect([
              recountOf({ group: groupOf({ outcomes: ['untouched', 'untouched',], carries: [], winner: none, },), },),
              recountOf({ group: groupOf({ outcomes: ['chosen', 'untouched',], carries: [], winner: none, },), },),
            ],).toEqual([
              'holds',
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[0].outcome: `
                + 'expected untouched, as the ballots and the renderings give for this place.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A RECEIVING PLACE WITH NO CARRY RECORDED, naming the place the chosen words are carried to',
          fn: async () => {
            expect(recountOf({
              group: groupOf({ outcomes: ['chosen', 'replaced',], carries: [], winner: FIRST_WON, },),
            },),).toBe(
              `ArtifactParseError: artifact parse failed at ${GROUP}.carries: `
                + 'expected a carry for OCCURRENCE 2, which the chosen words are carried to.',
            );
          },
        },),
      ],
    },),
  ],
},);
