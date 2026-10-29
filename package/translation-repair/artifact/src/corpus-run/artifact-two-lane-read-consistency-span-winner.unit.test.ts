/**
 Tests for the winner a recorded group of the sixteenth shape must have: the
 strict majority of the first round, or the winner of the run-off that settled a
 split (a tie for first, or a lone leader with the places next behind it); and the refusal of a run-off the ballots do not
 call for.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ConsistencyRendering, } from '@monochromatic-dev/translation-repair-consistency';
import {
  type ArtifactConsistencyBallot,
  type ArtifactConsistencyGroup,
  type ArtifactConsistencyRunOff,
  expectedWinnerOf,
  sameWinner,
} from '../../dist/final/node/index.mjs';

/**
 The group the cases read: a repeated sentence in two places.
 */
type AskedGroup = ArtifactConsistencyGroup & { readonly verdict: { readonly kind: 'asked'; }; };

/**
 Where the group sits in the section, for the refusals.
 */
const GROUP = 'cat.consistency.groups[0]';

/**
 Both places located, as the counter holds them.
 */
const RENDERINGS: readonly ConsistencyRendering[] = [
  { kind: 'located', quote: 'The cat naps', core: 'The cat naps', start: 0, },
  { kind: 'located', quote: 'the kitten naps', core: 'the kitten naps', start: 6, },
];

/**
 Three places located, as the counter holds them.
 */
const THREE_RENDERINGS: readonly ConsistencyRendering[] = [
  ...RENDERINGS,
  { kind: 'located', quote: 'The cat dozes', core: 'The cat dozes', start: 0, },
];

/**
 One ballot choosing a place and quoting both.

 @param seat - seat that answered

 @param choice - label of the place chosen

 @returns Ballot

 @example
 ```ts
 const ballot = ballotOf({ seat: 'hyper-cat', choice: 'OCCURRENCE 1', },);
 ```
 */
function ballotOf({ seat, choice, }: { readonly seat: string; readonly choice: string; },): ArtifactConsistencyBallot {
  return {
    seat,
    reason: 'It words the original best.',
    quotes: [
      { label: 'OCCURRENCE 1', quote: 'The cat naps', },
      { label: 'OCCURRENCE 2', quote: 'the kitten naps', },
    ],
    choice,
    keep: [],
  };
}

/**
 A group the bench was asked about, with the choices and the run-off a case gives.

 @param choices - label each seat chose, in order

 @param runOff - run-off recorded, if any

 @returns Group

 @example
 ```ts
 const group = groupOf({ choices: ['OCCURRENCE 1', 'OCCURRENCE 2',], },);
 ```
 */
function groupOf(
  {
    choices,
    runOff,
  }: {
    readonly choices: readonly string[];
    readonly runOff?: ArtifactConsistencyRunOff;
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
        outcome: 'chosen',
      },
      {
        label: 'OCCURRENCE 2',
        sliceIndex: 1,
        startOffset: 30,
        endOffset: 42,
        line: 5,
        rendering: { kind: 'located', quote: 'the kitten naps', start: 6, },
        outcome: 'replaced',
      },
    ],
    unplaced: [],
    verdict: {
      kind: 'asked',
      ballots: choices.map(function ballot(choice, at,): ArtifactConsistencyBallot {
        return ballotOf({ seat: `cat-${String(at,)}`, choice, },);
      },),
      usable: choices.length,
      quorum: 3,
      winner: { kind: 'none', reason: 'no-majority', },
      ...((runOff === undefined) ? {} : { runOff, }),
    },
  };
}

/**
 A group of three places, with the choices and the run-off a case gives.

 @param choices - label each seat chose, in order

 @param runOff - run-off recorded, if any

 @returns Group

 @example
 ```ts
 const group = threePlaceGroupOf({ choices: ['OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',], },);
 ```
 */
function threePlaceGroupOf(
  {
    choices,
    runOff,
  }: {
    readonly choices: readonly string[];
    readonly runOff?: ArtifactConsistencyRunOff;
  },
): AskedGroup {
  const group = groupOf({ choices, ...((runOff === undefined) ? {} : { runOff, }), },);
  return {
    ...group,
    occurrences: [
      ...group.occurrences,
      {
        label: 'OCCURRENCE 3',
        sliceIndex: 2,
        startOffset: 54,
        endOffset: 66,
        line: 7,
        rendering: { kind: 'located', quote: 'The cat dozes', start: 0, },
        outcome: 'replaced',
      },
    ],
  };
}

/**
 A run-off between the three places.

 @param choices - label each seat chose, in order

 @returns Run-off, with a quorum of the number of seats

 @example
 ```ts
 const runOff = threeFinalistRunOffOf({ choices: ['OCCURRENCE 3', 'OCCURRENCE 3', 'OCCURRENCE 1',], },);
 ```
 */
function threeFinalistRunOffOf({ choices, }: { readonly choices: readonly string[]; },): ArtifactConsistencyRunOff {
  return { ...runOffOf({ choices, },), finalists: ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',], };
}

/**
 A run-off between the two places.

 @param choices - label each seat chose, in order

 @returns Run-off, with a quorum of the number of seats

 @example
 ```ts
 const runOff = runOffOf({ choices: ['OCCURRENCE 1', 'OCCURRENCE 1',], },);
 ```
 */
function runOffOf({ choices, }: { readonly choices: readonly string[]; },): ArtifactConsistencyRunOff {
  return {
    finalists: ['OCCURRENCE 1', 'OCCURRENCE 2',],
    ballots: choices.map(function ballot(choice, at,): ArtifactConsistencyRunOff['ballots'][number] {
      return { seat: `cat-${String(at,)}`, reason: 'It words the original best.', choice, };
    },),
    usable: choices.length,
    quorum: choices.length,
  };
}

/**
 The text of the refusal the expected winner raises for a group.

 @param group - group, asked of the bench

 @returns Refusal as text

 @example
 ```ts
 const text = winnerRefusal({ group: groupOf({ choices: [], },), },);
 ```
 */
function winnerRefusal({ group, }: { readonly group: AskedGroup; },): string {
  return String(caught(function recounts(): unknown {
    return expectedWinnerOf({ group, renderings: RENDERINGS, path: GROUP, generation: 18, },);
  },),);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: sameWinner.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'AGREES WHEN BOTH NAME THE SAME PLACE OR GIVE THE SAME REASON FOR NONE, and not across the two kinds',
          fn: async () => {
            expect([
              sameWinner({ left: { kind: 'chosen', label: 'OCCURRENCE 1', }, right: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
              sameWinner({ left: { kind: 'chosen', label: 'OCCURRENCE 1', }, right: { kind: 'chosen', label: 'OCCURRENCE 2', }, },),
              sameWinner({ left: { kind: 'none', reason: 'no-quorum', }, right: { kind: 'none', reason: 'no-quorum', }, },),
              sameWinner({ left: { kind: 'none', reason: 'no-quorum', }, right: { kind: 'none', reason: 'run-off-tied', }, },),
              sameWinner({ left: { kind: 'none', reason: 'no-quorum', }, right: { kind: 'chosen', label: 'OCCURRENCE 1', }, },),
              sameWinner({ left: { kind: 'chosen', label: 'OCCURRENCE 1', }, right: { kind: 'none', reason: 'no-quorum', }, },),
            ],).toEqual([true, false, true, false, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: expectedWinnerOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES THE FIRST ROUND\'S MAJORITY WHEN THE CHOICES HAD NO FINALISTS, no majority when none had one, and no '
            + 'quorum when too few were usable',
          fn: async () => {
            expect([
              expectedWinnerOf({
                group: groupOf({ choices: ['OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 2',], }),
                renderings: RENDERINGS,
                path: GROUP,
                generation: 18,
              },),
              expectedWinnerOf({
                group: groupOf({ choices: ['OCCURRENCE 1', 'OCCURRENCE 1',], }),
                renderings: RENDERINGS,
                path: GROUP,
                generation: 18,
              },),
            ],).toEqual([
              { kind: 'chosen', label: 'OCCURRENCE 1', },
              { kind: 'none', reason: 'no-quorum', },
            ],);
          },
        },),
        it({
          name: 'GIVES THE RUN-OFF\'S WINNER WHEN THE CHOICES TIED FOR FIRST, and run-off-tied or run-off-no-quorum when the run-off '
            + 'itself tied or fell short',
          fn: async () => {
            const tied = ['OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 2',];
            expect([
              expectedWinnerOf({
                group: groupOf({ choices: tied, runOff: runOffOf({ choices: ['OCCURRENCE 2', 'OCCURRENCE 2', 'OCCURRENCE 1',], }), }),
                renderings: RENDERINGS,
                path: GROUP,
                generation: 18,
              },),
              expectedWinnerOf({
                group: groupOf({ choices: tied, runOff: runOffOf({ choices: ['OCCURRENCE 1', 'OCCURRENCE 2',], }), }),
                renderings: RENDERINGS,
                path: GROUP,
                generation: 18,
              },),
              expectedWinnerOf({
                group: groupOf({
                  choices: tied,
                  runOff: { ...runOffOf({ choices: ['OCCURRENCE 1', 'OCCURRENCE 1',], }), quorum: 3, },
                }),
                renderings: RENDERINGS,
                path: GROUP,
                generation: 18,
              },),
            ],).toEqual([
              { kind: 'chosen', label: 'OCCURRENCE 2', },
              { kind: 'none', reason: 'run-off-tied', },
              { kind: 'none', reason: 'run-off-no-quorum', },
            ],);
          },
        },),
        it({
          name: 'GIVES THE RUN-OFF\'S WINNER WHEN ONE PLACE LED ALONE WITHOUT A MAJORITY, the run-off among the leader and the '
            + 'places next behind it, and the page as it stands when that run-off splits again',
          fn: async () => {
            const split = ['OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',];
            expect([
              expectedWinnerOf({
                group: threePlaceGroupOf({
                  choices: split,
                  runOff: threeFinalistRunOffOf({ choices: ['OCCURRENCE 3', 'OCCURRENCE 3', 'OCCURRENCE 1',], }),
                }),
                renderings: THREE_RENDERINGS,
                path: GROUP,
                generation: 18,
              },),
              expectedWinnerOf({
                group: threePlaceGroupOf({
                  choices: split,
                  runOff: threeFinalistRunOffOf({ choices: ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',], }),
                }),
                renderings: THREE_RENDERINGS,
                path: GROUP,
                generation: 18,
              },),
            ],).toEqual([
              { kind: 'chosen', label: 'OCCURRENCE 3', },
              { kind: 'none', reason: 'run-off-tied', },
            ],);
          },
        },),
        it({
          name: 'REFUSES A LONE LEADER\'S SPLIT WITH NO RUN-OFF, AND ONE RUN OFF AMONG FEWER PLACES THAN THE FINALISTS',
          fn: async () => {
            const split = ['OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',];
            const refusal = `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.runOff: `
              + 'expected a run-off among OCCURRENCE 1, OCCURRENCE 2, OCCURRENCE 3, the finalists of ballots with no strict majority.';
            expect([
              threePlaceGroupOf({ choices: split, }),
              threePlaceGroupOf({ choices: split, runOff: runOffOf({ choices: ['OCCURRENCE 1', 'OCCURRENCE 2',], }), }),
            ].map(function refusedFor(group,): string {
              return String(caught(function recounts(): unknown {
                return expectedWinnerOf({ group, renderings: THREE_RENDERINGS, path: GROUP, generation: 18, },);
              },),);
            },),).toEqual([refusal, refusal,],);
          },
        },),
        it({
          name: 'REFUSES A RUN-OFF RECORDED FOR CHOICES THAT HAD NO FINALISTS, A TIE WITH NO RUN-OFF AND A RUN-OFF AMONG OTHER FINALISTS',
          fn: async () => {
            const tied = ['OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 2',];
            expect([
              groupOf({
                choices: ['OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 1',],
                runOff: runOffOf({ choices: ['OCCURRENCE 1', 'OCCURRENCE 1',], }),
              }),
              groupOf({ choices: tied, }),
              groupOf({
                choices: tied,
                runOff: { ...runOffOf({ choices: ['OCCURRENCE 1', 'OCCURRENCE 1',], }), finalists: ['OCCURRENCE 1',], },
              }),
            ].map(function refusedFor(group,): string {
              return winnerRefusal({ group, },);
            },),).toEqual([
              `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.runOff: `
                + 'expected no run-off, since the usable ballots left no finalists.',
              `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.runOff: `
                + 'expected a run-off among OCCURRENCE 1, OCCURRENCE 2, the finalists of ballots with no strict majority.',
              `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.runOff: `
                + 'expected a run-off among OCCURRENCE 1, OCCURRENCE 2, the finalists of ballots with no strict majority.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A RUN-OFF WITH A QUORUM BELOW ONE, A SEAT HEARD TWICE OR A USABLE COUNT THAT IS NOT ITS BALLOTS\'',
          fn: async () => {
            const tied = ['OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 2',];
            const base = runOffOf({ choices: ['OCCURRENCE 1', 'OCCURRENCE 2',], });
            const first = nonNullishOrThrow(base.ballots[0],);
            const variants: readonly ArtifactConsistencyRunOff[] = [
              { ...base, quorum: 0, },
              { ...base, ballots: [first, first,], },
              { ...base, usable: 1, },
            ];
            expect(variants.map(function refusedFor(runOff,): string {
              return winnerRefusal({ group: groupOf({ choices: tied, runOff, }), },);
            },),).toEqual([
              `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.runOff: `
                + 'expected a quorum of one at least, each seat heard once, and the number of ballots that can be counted '
                + 'against the finalists.',
              `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.runOff: `
                + 'expected a quorum of one at least, each seat heard once, and the number of ballots that can be counted '
                + 'against the finalists.',
              `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.runOff: `
                + 'expected a quorum of one at least, each seat heard once, and the number of ballots that can be counted '
                + 'against the finalists.',
            ],);
          },
        },),
      ],
    },),
  ],
},);
