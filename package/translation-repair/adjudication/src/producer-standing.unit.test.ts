/**
 Tests for the per-producer standing: what counts as a disinterested ballot,
 who a candidate credits, and what an empty denominator means.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { SelectionBallot, } from '@monochromatic-dev/translation-repair-house-rule';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  preferenceRate,
  type ProducerStanding,
  producerStandings,
} from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Full weight a ballot carries when the judge holds no stake.
 */
const FULL = 1;

/**
 Builds one ballot with the fields the standing reads.

 @param modelId - judge casting it

 @param best - one-based candidate it named

 @param selfVote - whether it named text it produced; the standing does not
 read this, and a case proving that passes it deliberately wrong

 @returns Ballot for a selection round

 @example
 ```ts
 const ballot = ballotOf({ modelId: SEAT_HYPER_VISION, best: 1, },);
 ```
 */
function ballotOf(
  {
    modelId,
    best,
    selfVote = false,
  }: {
    readonly modelId: RosterModelId;
    readonly best: number;
    readonly selfVote?: boolean;
  },
): SelectionBallot {
  return {
    modelId,
    best,
    reason: 'the cat sits better in this one',
    weight: FULL,
    selfVote,
  };
}

/**
 Reads one model's standing out of a list, refusing an absent one.

 @param standings - what the tally produced

 @param modelId - model wanted

 @returns That model's counts

 @example
 ```ts
 const standing = standingOf({ standings, modelId: SEAT_HYPER_VISION, },);
 ```
 */
function standingOf(
  {
    standings,
    modelId,
  }: {
    readonly standings: readonly ProducerStanding[];
    readonly modelId: RosterModelId;
  },
): ProducerStanding {
  /**
   Entry for that model, absent when it wrote nothing.
   */
  const found = standings.find(function named(standing,): boolean {
    return standing.modelId === modelId;
  },);
  if (found === undefined)
    throw new Error(`no standing for ${modelId}`,);
  return found;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: producerStandings.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES to count a producer\'s vote for its own candidate',
          fn: async () => {
            /** One slate of two candidates, each author voting for itself. */
            const rounds: readonly SelectionRound[] = [
              {
                producers: [
                  { kind: 'model' as const, modelId: SEAT_HYPER_VISION, },
                  { kind: 'model' as const, modelId: SEAT_BEDROCK_ONLY_VISION_UNSEATED, },
                ],
                ballots: [
                  ballotOf({ modelId: SEAT_HYPER_VISION, best: 1, },),
                  ballotOf({ modelId: SEAT_BEDROCK_ONLY_VISION_UNSEATED, best: 2, },),
                ],
              },
            ];

            /** What the tally made of it. */
            const standings = producerStandings({ rounds, },);

            // Counting self-votes would rank the most self-confident model first
            // rather than the best-written one, and seat exactly the models least
            // able to tell.
            for (const modelId of [SEAT_HYPER_VISION, SEAT_BEDROCK_ONLY_VISION_UNSEATED,] as const) {
              /** That model's counts. */
              const standing = standingOf({
                standings,
                modelId,
              },);

              expect(standing.candidates,).toBe(1,);
              expect(standing.disinterestedVotes,).toBe(0,);
              // One ballot each, from the other author, naming the other candidate.
              expect(standing.disinterestedBallots,).toBe(1,);
            }
          },
        },),

        it({
          name: 'counts a vote from a judge holding no stake in that candidate',
          fn: async () => {
            /** Two candidates, and a third model judging both. */
            const rounds: readonly SelectionRound[] = [
              {
                producers: [
                  { kind: 'model' as const, modelId: SEAT_HYPER_VISION, },
                  { kind: 'model' as const, modelId: SEAT_BEDROCK_ONLY_VISION_UNSEATED, },
                ],
                ballots: [
                  ballotOf({ modelId: SEAT_HYPER_TEXT_BEDROCK, best: 1, },),
                ],
              },
            ];

            /** What the tally made of it. */
            const standings = producerStandings({ rounds, },);

            /** The named candidate's author. */
            const named = standingOf({
              standings,
              modelId: SEAT_HYPER_VISION,
            },);

            /** The passed-over candidate's author. */
            const passed = standingOf({
              standings,
              modelId: SEAT_BEDROCK_ONLY_VISION_UNSEATED,
            },);

            expect(named.disinterestedVotes,).toBe(1,);
            expect(passed.disinterestedVotes,).toBe(0,);
            // Both candidates were judged by the same disinterested voice.
            expect(passed.disinterestedBallots,).toBe(1,);
          },
        },),

        it({
          name: 'credits every contributor to a composite, since it is joint work',
          fn: async () => {
            /** One composite candidate and one plain one. */
            const rounds: readonly SelectionRound[] = [
              {
                producers: [
                  {
                    kind: 'composite' as const,
                    contributors: [SEAT_HYPER_VISION, SEAT_BEDROCK_ONLY_VISION_UNSEATED,],
                  },
                  { kind: 'model' as const, modelId: SEAT_HYPER_TEXT_BEDROCK, },
                ],
                ballots: [
                  ballotOf({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, best: 1, },),
                ],
              },
            ];

            /** What the tally made of it. */
            const standings = producerStandings({ rounds, },);

            for (const modelId of [SEAT_HYPER_VISION, SEAT_BEDROCK_ONLY_VISION_UNSEATED,] as const) {
              /** That contributor's counts. */
              const standing = standingOf({
                standings,
                modelId,
              },);

              expect(standing.disinterestedVotes,).toBe(1,);
            }
          },
        },),

        it({
          name: 'REFUSES to credit anyone for an incumbent nobody matched',
          fn: async () => {
            /** The archive's own text, standing alone. */
            const rounds: readonly SelectionRound[] = [
              {
                producers: [
                  { kind: 'incumbent' as const, matched: [], },
                  { kind: 'model' as const, modelId: SEAT_HYPER_VISION, },
                ],
                ballots: [
                  ballotOf({ modelId: SEAT_HYPER_TEXT_BEDROCK, best: 1, },),
                ],
              },
            ];

            /** What the tally made of it. */
            const standings = producerStandings({ rounds, },);

            // The archive is not a roster member, so an incumbent standing alone
            // credits no one; that is correct rather than a gap.
            /** The only model that wrote anything here. */
            const standing = standingOf({
              standings,
              modelId: SEAT_HYPER_VISION,
            },);

            expect(standings.length,).toBe(1,);
            expect(standing.disinterestedVotes,).toBe(0,);
          },
        },),

        it({
          name: 'credits the models collapsed into an incumbent, which did write it',
          fn: async () => {
            /** An incumbent that one model independently reproduced. */
            const rounds: readonly SelectionRound[] = [
              {
                producers: [
                  { kind: 'incumbent' as const, matched: [SEAT_HYPER_VISION,], },
                ],
                ballots: [
                  ballotOf({ modelId: SEAT_HYPER_TEXT_BEDROCK, best: 1, },),
                  ballotOf({ modelId: SEAT_HYPER_VISION, best: 1, },),
                ],
              },
            ];

            /** What the tally made of it. */
            const standings = producerStandings({ rounds, },);

            /** The matched model's counts. */
            const standing = standingOf({
              standings,
              modelId: SEAT_HYPER_VISION,
            },);

            // Its own ballot is a self-vote and is not counted, so one of the two
            // ballots survives.
            expect(standing.disinterestedBallots,).toBe(1,);
            expect(standing.disinterestedVotes,).toBe(1,);
          },
        },),

        it({
          name: 'accumulates across rounds rather than reporting the last one',
          fn: async () => {
            /** Two slates naming the same two authors. */
            const slate: SelectionRound['producers'] = [
              { kind: 'model' as const, modelId: SEAT_HYPER_VISION, },
              { kind: 'model' as const, modelId: SEAT_BEDROCK_ONLY_VISION_UNSEATED, },
            ];
            /** Rounds with a disinterested judge splitting its votes. */
            const rounds: readonly SelectionRound[] = [
              {
                producers: slate,
                ballots: [ballotOf({ modelId: SEAT_HYPER_TEXT_BEDROCK, best: 1, },),],
              },
              {
                producers: slate,
                ballots: [ballotOf({ modelId: SEAT_HYPER_TEXT_BEDROCK, best: 2, },),],
              },
            ];

            /** What the tally made of it. */
            const standings = producerStandings({ rounds, },);

            for (const modelId of [SEAT_HYPER_VISION, SEAT_BEDROCK_ONLY_VISION_UNSEATED,] as const) {
              /** That model's counts. */
              const standing = standingOf({
                standings,
                modelId,
              },);

              expect(standing.candidates,).toBe(2,);
              expect(standing.disinterestedBallots,).toBe(2,);
              expect(standing.disinterestedVotes,).toBe(1,);
            }
          },
        },),

        it({
          name: 'REFUSES to read the ballot\'s own self-vote flag instead of the stake',
          fn: async () => {
            // A judge with a stake in candidate 1 that votes for candidate 2
            // carries `selfVote: false`, and its opinion of its own rival is still
            // not disinterested. Reading the flag would admit that ballot.
            const rounds: readonly SelectionRound[] = [
              {
                producers: [
                  { kind: 'model' as const, modelId: SEAT_HYPER_VISION, },
                  { kind: 'model' as const, modelId: SEAT_BEDROCK_ONLY_VISION_UNSEATED, },
                ],
                ballots: [
                  ballotOf({ modelId: SEAT_HYPER_VISION, best: 2, selfVote: false, },),
                ],
              },
            ];

            /** What the tally made of it. */
            const standings = producerStandings({ rounds, },);

            /** The voter's own candidate. */
            const own = standingOf({
              standings,
              modelId: SEAT_HYPER_VISION,
            },);

            /** The rival's candidate, which the voter named. */
            const rival = standingOf({
              standings,
              modelId: SEAT_BEDROCK_ONLY_VISION_UNSEATED,
            },);

            // ITS OWN CANDIDATE sees no disinterested ballot: the only voice here
            // holds a stake in it. Reading `selfVote` instead would admit this
            // ballot, because the judge did not name its own work, and inflate its
            // author's denominator with its author's own ballot.
            expect(own.disinterestedBallots,).toBe(0,);

            // THE RIVAL'S CANDIDATE keeps the vote, and should: a competitor
            // naming someone else's work is the strongest disinterested signal
            // this measurement can get.
            expect(rival.disinterestedBallots,).toBe(1,);
            expect(rival.disinterestedVotes,).toBe(1,);
          },
        },),

        it({
          name: 'REFUSES to count an abstention as a vote for anyone',
          fn: async () => {
            /** One slate where the only judge named nothing. */
            const rounds: readonly SelectionRound[] = [
              {
                producers: [{ kind: 'model' as const, modelId: SEAT_HYPER_VISION, },],
                ballots: [ballotOf({ modelId: SEAT_HYPER_TEXT_BEDROCK, best: 0, },),],
              },
            ];

            /** The producer's counts. */
            const standing = standingOf({
              standings: producerStandings({ rounds, },),
              modelId: SEAT_HYPER_VISION,
            },);

            expect(standing.disinterestedBallots,).toBe(1,);
            expect(standing.disinterestedVotes,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: preferenceRate.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'divides votes by the ballots that could have been cast',
          fn: async () => {
            expect(preferenceRate({
              standing: {
                modelId: SEAT_HYPER_VISION,
                candidates: 4,
                disinterestedBallots: 8,
                disinterestedVotes: 2,
              },
            },),).toEqual({ measured: true, share: 0.25, },);
          },
        },),

        it({
          name: 'REFUSES to report a share nobody voted on',
          fn: async () => {
            // A zero here would sort this model below every measured one and read
            // as the strongest evidence against it, which is the opposite of what
            // an empty denominator means.
            expect(preferenceRate({
              standing: {
                modelId: SEAT_HYPER_VISION,
                candidates: 2,
                disinterestedBallots: 0,
                disinterestedVotes: 0,
              },
            },),).toEqual({ measured: false, },);
          },
        },),
      ],
    },),
  ],
},);
