/**
 Guards class sixty-three (XingZ613, 2026-09-19): a slate over an ineligible
 standing whose judges split a plurality under the minimum (1.5, 1, 1 of a
 bench at quorum) was challenged as a run-off over every candidate that drew
 a ballot, which was the whole slate again, and it split the same way; the
 entry stopped over a heading whose every rendering was valid. The run-off
 narrows to the leaders: the tied leaders on a tie, the leader and its
 runner-up on a plurality, so the challenge is a question this bench can
 settle. Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  Candidate,
  CandidateWeight,
} from '@monochromatic-dev/translation-repair-house-rule';
import { runoffFinalists, } from '../dist/final/node/index.mjs';

/**
 Renderings on the slate, one per translator, in slate order.
 */
const RENDERINGS: readonly string[] = [
  'Part Nine: the cat on the sill.',
  'The Ninth: the cat on the sill.',
  '其九: the cat on the sill.',
  'Chapter nine: the cat on the sill.',
];

/**
 Slate of the first `count` renderings.

 @param count - how many candidates the slate offers

 @returns Candidates in slate order

 @example
 ```ts
 const slate = candidates({ count: 4, },);
 ```
 */
function candidates({ count, }: { readonly count: number; },): readonly Candidate<string>[] {
  return RENDERINGS.slice(
    0,
    count,
  ).map(function toCandidate(
    rendered,
    position,
  ): Candidate<string> {
    return {
      producer: {
        kind: 'model',
        modelId: `hf:cat/Cat-${String(position + 1,)}` as unknown as RosterModelId,
      },
      value: rendered,
      rendered,
    };
  },);
}

/**
 What one candidate drew, from its full-weight ballots alone.

 @param index - one-based candidate index

 @param ballots - ballots naming it

 @param weight - summed weight those ballots carried

 @returns Per-candidate count as the round reports it

 @example
 ```ts
 const drawn = drew({ index: 4, ballots: 2, weight: 1.5, },);
 ```
 */
function drew(
  {
    index,
    ballots,
    weight,
  }: {
    readonly index: number;
    readonly ballots: number;
    readonly weight: number;
  },
): CandidateWeight {
  return {
    index,
    ballots,
    fullVotes: ballots,
    selfVotes: 0,
    weight,
  };
}

/**
 Slate of four, which the whole-value checks name finalists from.
 */
const SLATE_OF_FOUR = candidates({ count: 4, },);

/**
 Slate of three, which the whole-value checks name finalists from.
 */
const SLATE_OF_THREE = candidates({ count: 3, },);

await describe({
  name: 'the run-off narrows to the leaders (class sixty-three, XingZ613 slice 84)',
  children: [
    it({
      name: 'OFFERS the leader and its runner-up on a plurality under the minimum',
      fn: async () => {
        // XingZ613's first round: candidate 4 at 1.5 from two ballots,
        // candidates 1 and 3 at 1 from one ballot each, candidate 2 unnamed.
        const runoff = runoffFinalists({
          candidates: candidates({ count: 4, },),
          perCandidate: [
            drew({
              index: 1,
              ballots: 1,
              weight: 1,
            },),
            drew({
              index: 3,
              ballots: 1,
              weight: 1,
            },),
            drew({
              index: 4,
              ballots: 2,
              weight: 1.5,
            },),
          ],
          disposition: 'indecision',
        },);
        expect(runoff,).toEqual({
          kind: 'narrowed',
          basis: 'leader-and-runner-up',
          finalists: [
            SLATE_OF_FOUR[0],
            SLATE_OF_FOUR[3],
          ],
        },);
      },
    },),
    it({
      name: 'OFFERS only the tied leaders when a lower candidate also drew a ballot',
      fn: async () => {
        const runoff = runoffFinalists({
          candidates: candidates({ count: 4, },),
          perCandidate: [
            drew({
              index: 1,
              ballots: 1,
              weight: 1,
            },),
            drew({
              index: 2,
              ballots: 1,
              weight: 1,
            },),
            drew({
              index: 4,
              ballots: 1,
              weight: 0.5,
            },),
          ],
          disposition: 'indecision',
        },);
        expect(runoff,).toEqual({
          kind: 'narrowed',
          basis: 'tied-at-top',
          finalists: [
            SLATE_OF_FOUR[0],
            SLATE_OF_FOUR[1],
          ],
        },);
      },
    },),
    it({
      name: 'KEEPS the class fifty-three run-off: two candidates named once each of three',
      fn: async () => {
        const runoff = runoffFinalists({
          candidates: candidates({ count: 3, },),
          perCandidate: [
            drew({
              index: 1,
              ballots: 1,
              weight: 1,
            },),
            drew({
              index: 2,
              ballots: 1,
              weight: 1,
            },),
          ],
          disposition: 'indecision',
        },);
        expect(runoff,).toEqual({
          kind: 'narrowed',
          basis: 'tied-at-top',
          finalists: [
            SLATE_OF_THREE[0],
            SLATE_OF_THREE[1],
          ],
        },);
      },
    },),
    it({
      name: 'PREFERS THE RUNNER-UP WITH MORE BALLOTS at equal weight, before slate position',
      fn: async () => {
        // Candidate 1 leads; candidates 2 and 3 both drew a weight of 1, one
        // from a full ballot and one from two half ballots.
        expect(runoffFinalists({
          candidates: SLATE_OF_FOUR,
          perCandidate: [
            drew({
              index: 1,
              ballots: 2,
              weight: 2,
            },),
            drew({
              index: 2,
              ballots: 1,
              weight: 1,
            },),
            drew({
              index: 3,
              ballots: 2,
              weight: 1,
            },),
          ],
          disposition: 'indecision',
        },),).toEqual({
          kind: 'narrowed',
          basis: 'leader-and-runner-up',
          finalists: [
            SLATE_OF_FOUR[0],
            SLATE_OF_FOUR[2],
          ],
        },);
      },
    },),
    it({
      name: 'OFFERS the whole slate again when every candidate tied at the top',
      fn: async () => {
        const runoff = runoffFinalists({
          candidates: candidates({ count: 3, },),
          perCandidate: [
            drew({
              index: 1,
              ballots: 1,
              weight: 1,
            },),
            drew({
              index: 2,
              ballots: 1,
              weight: 1,
            },),
            drew({
              index: 3,
              ballots: 1,
              weight: 1,
            },),
          ],
          disposition: 'indecision',
        },);
        expect(runoff,).toEqual({
          kind: 'whole-slate',
          because: 'every-candidate-backed',
        },);
      },
    },),
    it({
      name: 'OFFERS the whole slate again on a rejection',
      fn: async () => {
        const runoff = runoffFinalists({
          candidates: candidates({ count: 3, },),
          perCandidate: [],
          disposition: 'rejection',
        },);
        expect(runoff,).toEqual({
          kind: 'whole-slate',
          because: 'rejection',
        },);
      },
    },),
  ],
},);
