/**
 Tests for projecting the repair lane's rounds into the shape a producer
 standing counts.

 THE POSITION GUARD IS THE POINT. A ballot names a candidate by number, so
 the projection's whole correctness rests on slate order matching the numbers
 judges were shown. A slate that disagrees must refuse rather than quietly
 credit the wrong model, and that refusal is exercised in this file.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { producerStandings, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  EDITOR_ROUND_STAGES,
  REFINER_ROUND_STAGES,
  type RepairJudgedRound,
  type RepairSlateEntry,
  selectionRoundOf,
  selectionRoundsFor,
  SlatePositionsError,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Builds one slate entry at a stated position.

 @param index - one-based position judges were shown

 @param modelId - model that wrote it

 @returns Slate entry

 @example
 ```ts
 const entry = entryAt({ index: 1, modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, },);
 ```
 */
function entryAt(
  {
    index,
    modelId,
  }: {
    readonly index: number;
    readonly modelId: RosterModelId;
  },
): RepairSlateEntry {
  return {
    index,
    rendered: `candidate ${String(index,)}`,
    hash: `hash-${String(index,)}`,
    producer: {
      kind: 'model' as const,
      modelId,
    },
  };
}

/**
 A selected round whose slate is in scrambled order, to prove the projection
 sorts rather than trusts.
 */
const SCRAMBLED_ROUND: RepairJudgedRound = {
  kind: 'selected' as const,
  stage: 'envelope' as const,
  envelopeId: 'env-1',
  slate: [
    entryAt({
      index: 2,
      modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    },),
    entryAt({
      index: 1,
      modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
    },),
  ],
  ballots: [
    {
      modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
      best: 1,
      reason: 'reads better',
      weight: 1,
      selfVote: false,
    },
  ],
  tally: {
    judgesAvailable: 1,
    ballots: 1,
    abstentions: 0,
    selfVotes: 0,
  },
  perCandidate: [],
  selectedIndex: 1,
  voteWeight: 1,
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: selectionRoundOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'orders producers by the position judges were shown, not by slate order',
          fn: async () => {
            /**
             Projection of a slate recorded out of order.
             */
            const projected = selectionRoundOf({ round: SCRAMBLED_ROUND, },);

            // Position 1 was Kimi, so it must land first however the slate was stored.
            expect(projected.producers[0],).toEqual({
              kind: 'model',
              modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
            },);
            expect(projected.producers[1],).toEqual({
              kind: 'model',
              modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            },);
          },
        },),

        it({
          name: 'FORWARDS every ballot unchanged',
          fn: async () => {
            expect(selectionRoundOf({ round: SCRAMBLED_ROUND, },).ballots,)
              .toEqual(SCRAMBLED_ROUND.ballots,);
          },
        },),

        it({
          name: 'REFUSES a slate whose positions are not one to its length',
          fn: async () => {
            /**
             A slate that skips position two, which a ballot naming two would
             silently misread as the third candidate.
             */
            const gapped: RepairJudgedRound = {
              ...SCRAMBLED_ROUND,
              slate: [
                entryAt({
                  index: 1,
                  modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
                },),
                entryAt({
                  index: 3,
                  modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                },),
              ],
            };

            expect(() => selectionRoundOf({ round: gapped, },),).toThrow(SlatePositionsError,);
          },
        },),
      ],
    },),

    describe({
      name: selectionRoundsFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'keeps only the stages the named role produced',
          fn: async () => {
            /**
             One round from each stage the lane records.
             */
            const rounds: readonly RepairJudgedRound[] = [
              SCRAMBLED_ROUND,
              {
                ...SCRAMBLED_ROUND,
                stage: 'chunk-patch' as const,
              },
              {
                ...SCRAMBLED_ROUND,
                stage: 'refine' as const,
              },
            ];

            expect(selectionRoundsFor({
              rounds,
              stages: EDITOR_ROUND_STAGES,
            },).length,).toBe(2,);

            expect(selectionRoundsFor({
              rounds,
              stages: REFINER_ROUND_STAGES,
            },).length,).toBe(1,);
          },
        },),

        it({
          name: 'ACCEPTS a declined round, because its ballots are evidence too',
          fn: async () => {
            /**
             A round where judges saw the slate and settled on nothing.
             */
            const declined: RepairJudgedRound = {
              kind: 'declined' as const,
              stage: 'envelope' as const,
              envelopeId: 'env-2',
              slate: SCRAMBLED_ROUND.slate,
              ballots: SCRAMBLED_ROUND.ballots,
              tally: {
                judgesAvailable: 1,
                ballots: 1,
                abstentions: 0,
                selfVotes: 0,
              },
              perCandidate: [],
              reason: 'no candidate drew a majority',
              disposition: 'indecision' as const,
            };

            expect(selectionRoundsFor({
              rounds: [declined,],
              stages: EDITOR_ROUND_STAGES,
            },).length,).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: 'editor standing over projected rounds',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS real-shaped rounds and produces a standing with counts',
          fn: async () => {
            /**
             A POSITIVE CONTROL FOR THE WHOLE READING CHAIN.

             The first live smoke run reported zero rounds, correctly: its slice
             carried no accepted issue, so no editor was ever asked to write. A
             null from a probe never shown able to produce a non-null says
             nothing, so this drives projection and tally together and checks a
             standing actually falls out with the counts behind it.

             Kimi wrote position one and Qwen position two. Three judges hold no
             stake in either, and all three named position one.
             */
            const rounds = [
              {
                ...SCRAMBLED_ROUND,
                ballots: [
                  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  SEAT_HYPER_OPENROUTER_UNMEASURED,
                ].map(function ballotFor(modelId,) {
                  return {
                    modelId: modelId as RosterModelId,
                    best: 1,
                    reason: 'reads better',
                    weight: 1,
                    selfVote: false,
                  };
                },),
              },
            ] satisfies readonly RepairJudgedRound[];

            /**
             What the editors' rounds came to.
             */
            const standings = producerStandings({
              rounds: selectionRoundsFor({
                rounds,
                stages: EDITOR_ROUND_STAGES,
              },),
            },);

            /**
             Kimi's standing, which wrote the candidate all three named.
             */
            const kimi = standings.find(function isKimi(standing,): boolean {
              return standing.modelId === SEAT_SYNTHETIC_VISION_WITHHELD;
            },);

            expect(kimi?.disinterestedVotes,).toBe(3,);
            expect(kimi?.disinterestedBallots,).toBe(3,);
            expect(kimi?.candidates,).toBe(1,);

            /**
             Qwen's standing, which wrote the candidate none of them named.
             */
            const qwen = standings.find(function isQwen(standing,): boolean {
              return standing.modelId === SEAT_SYNTHETIC_VISION_NO_OPENROUTER;
            },);

            expect(qwen?.disinterestedVotes,).toBe(0,);
            expect(qwen?.disinterestedBallots,).toBe(3,);
          },
        },),
      ],
    },),
  ],
},);
