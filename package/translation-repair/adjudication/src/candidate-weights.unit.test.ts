/**
 Tests for what a candidate is recorded as having drawn.

 WHY THIS FILE EXISTS. A decline has to say by how much the leader fell short
 and against what, so every candidate gets a row even when nobody named it,
 and a row counts only ballots that actually carried weight. Measured on
 2026-08-25, counting weightless ballots as well failed no case.

 WHEN A WEIGHTLESS BALLOT CAN NAME A CANDIDATE. Not today: the producer sets
 `best` and weight together, so a ballot naming a candidate in range always
 carries `FULL_VOTE_WEIGHT` or `SELF_VOTE_WEIGHT`, and both are above zero.
 It becomes reachable the moment `SELF_VOTE_WEIGHT` is tuned to zero, which
 is a knob rather than a constant, and the counts would then credit a judge`s
 vote for its own work as a ballot while the weight it contributed stayed
 nothing. That is the reading this case pins.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { SelectionBallot, } from '@monochromatic-dev/translation-repair-house-rule';
import { countCandidateWeights, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Fixtures

/**
 Builds one ballot as the selection stage weighs them.

 @param modelId - judge that cast it

 @param best - one-based candidate index it named

 @param weight - weight it carried

 @param selfVote - whether that judge helped write what it named

 @returns Ballot shaped as the stage records one

 @example
 ```ts
 const ballot = ballotOf({ modelId, best: 1, weight: 1, selfVote: false, },);
 ```
 */
function ballotOf(
  {
    modelId,
    best,
    weight,
    selfVote,
  }: {
    readonly modelId: SelectionBallot['modelId'];
    readonly best: number;
    readonly weight: number;
    readonly selfVote: boolean;
  },
): SelectionBallot {
  return {
    modelId,
    best,
    reason: 'the cat reads more naturally there',
    weight,
    selfVote,
  };
}

//endregion Fixtures

await describe({
  name: countCandidateWeights.name,
  children: [
    it({
      name: 'REFUSES to count a ballot that named a candidate while carrying no weight, so a row can '
        + 'never report more ballots than the weight they contributed accounts for',
      fn: async () => {
        /**
         Two judges naming candidate one, of which only the first counted.
         */
        const rows = countCandidateWeights({
          ballots: [
            ballotOf({
              modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              best: 1,
              weight: 1,
              selfVote: false,
            },),
            ballotOf({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              best: 1,
              weight: 0,
              selfVote: true,
            },),
          ],
          candidateCount: 1,
        },);

        expect(rows,).toEqual([{
          index: 1,
          ballots: 1,
          fullVotes: 1,
          selfVotes: 0,
          weight: 1,
        },],);
      },
    },),
    it({
      name: 'KEEPS a row for a candidate nobody named, since a reader deciding whether a decline was '
        + 'close needs to tell a candidate that drew nothing from one that was never offered',
      fn: async () => {
        /**
         Three candidates, one of which drew a half-weight self-vote and one
         of which drew nothing at all.
         */
        const rows = countCandidateWeights({
          ballots: [
            ballotOf({
              modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              best: 1,
              weight: 1,
              selfVote: false,
            },),
            ballotOf({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              best: 2,
              weight: 1 / 2,
              selfVote: true,
            },),
          ],
          candidateCount: 3,
        },);

        expect(rows,).toEqual([
          {
            index: 1,
            ballots: 1,
            fullVotes: 1,
            selfVotes: 0,
            weight: 1,
          },
          {
            index: 2,
            ballots: 1,
            fullVotes: 0,
            selfVotes: 1,
            weight: 1 / 2,
          },
          {
            index: 3,
            ballots: 0,
            fullVotes: 0,
            selfVotes: 0,
            weight: 0,
          },
        ],);
      },
    },),
  ],
},);
