/**
 Tests for rendering and ordering producer standings.

 WHAT THESE PIN is the pair of decisions a reader of a calibration report has
 to trust: that a share always arrives with the denominator behind it, and
 that a model no disinterested judge ever voted on sorts to the END rather
 than to the bottom. The second is the one that changes conclusions. A model
 with no evidence and a model measured at zero are different findings, and a
 ranking that put them side by side would report the first as the second.

 Counts are invention. Model ids come from the catalog, since the standing
 type takes a roster id and a made-up one would not type.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  rankStandings,
  standingLine,
  type ProducerStanding,
} from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Builds one standing, so each case states only what it is about.

 @param modelId - roster model the standing describes

 @param candidates - slates carrying a candidate this model helped write

 @param disinterestedBallots - ballots cast over those by judges with no stake

 @param disinterestedVotes - how many of those named this model's candidate

 @returns Standing as the tally produces it

 @example
 ```ts
 const standing = standingOf({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, candidates: 4, disinterestedBallots: 8, disinterestedVotes: 6, },);
 ```
 */
function standingOf(
  {
    modelId,
    candidates,
    disinterestedBallots,
    disinterestedVotes,
  }: {
    readonly modelId: ProducerStanding['modelId'];
    readonly candidates: number;
    readonly disinterestedBallots: number;
    readonly disinterestedVotes: number;
  },
): ProducerStanding {
  return {
    modelId,
    candidates,
    disinterestedBallots,
    disinterestedVotes,
  };
}

/**
 Model that won most of the ballots cast over its candidates.
 */
const LEADER = standingOf({
  modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  candidates: 4,
  disinterestedBallots: 8,
  disinterestedVotes: 6,
},);

/**
 Model measured at zero, which is evidence rather than absence of it.
 */
const MEASURED_ZERO = standingOf({
  modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  candidates: 3,
  disinterestedBallots: 5,
  disinterestedVotes: 0,
},);

/**
 Model no disinterested judge ever voted on.
 */
const UNJUDGED = standingOf({
  modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
  candidates: 2,
  disinterestedBallots: 0,
  disinterestedVotes: 0,
},);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: standingLine.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'renders the share to one decimal AND carries the three counts behind it, since a share with '
              + 'no denominator beside it cannot be told apart from a share one ballot wide',
          fn: async () => {
            expect(standingLine({ standing: LEADER, },),).toBe(
              'hf:zai-org/GLM-5.3-Flash: 75.0% (6 of 8 disinterested ballots, over 4 candidates)',
            );
          },
        },),

        it({
          name:
            'says UNJUDGED rather than 0.0% for a model no disinterested judge voted on, which is the '
              + 'distinction the whole report exists to keep: no evidence is not evidence of a poor showing',
          fn: async () => {
            expect(standingLine({ standing: UNJUDGED, },),).toBe(
              'hf:moonshotai/Kimi-K3: UNJUDGED (0 of 0 disinterested ballots, over 2 candidates)',
            );
          },
        },),

        it({
          name:
            'renders a MEASURED zero as 0.0%, so the two zeroes a reader might confuse read differently on '
              + 'the page rather than only in the data behind it',
          fn: async () => {
            expect(standingLine({ standing: MEASURED_ZERO, },),).toBe(
              'hf:Qwen/Qwen3.8-27B: 0.0% (0 of 5 disinterested ballots, over 3 candidates)',
            );
            expect(standingLine({ standing: MEASURED_ZERO, },),).not.toBe(
              standingLine({ standing: UNJUDGED, },),
            );
          },
        },),
      ],
    },),

    describe({
      name: rankStandings.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'POSITIVE CONTROL: the input order is not already the answer, so a ranking that returned its '
              + 'argument untouched would fail the other cases in this file rather than pass them',
          fn: async () => {
            expect([
              UNJUDGED,
              MEASURED_ZERO,
              LEADER,
            ].map(function idOf(standing,): string {
              return standing.modelId;
            },),).not.toEqual(rankStandings({
              standings: [
                UNJUDGED,
                MEASURED_ZERO,
                LEADER,
              ],
            },).map(function idOf(standing,): string {
              return standing.modelId;
            },),);
          },
        },),

        it({
          name:
            'orders by share, best first, and puts the UNJUDGED model behind the one measured at zero: a '
              + 'model with no ballots sorts to the END, not to the bottom, because it wrote candidates '
              + 'nobody disinterested ever voted on',
          fn: async () => {
            expect(rankStandings({
              standings: [
                UNJUDGED,
                MEASURED_ZERO,
                LEADER,
              ],
            },).map(function idOf(standing,): string {
              return standing.modelId;
            },),).toEqual([
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              SEAT_SYNTHETIC_VISION_WITHHELD,
            ],);
          },
        },),

        it({
          name:
            'puts an UNJUDGED model that arrived after a measured one behind it and behind the model measured '
              + 'at zero that arrived last',
          fn: async () => {
            expect(rankStandings({
              standings: [
                LEADER,
                UNJUDGED,
                MEASURED_ZERO,
              ],
            },).map(function idOf(standing,): string {
              return standing.modelId;
            },),).toEqual([
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              SEAT_SYNTHETIC_VISION_WITHHELD,
            ],);
          },
        },),

        it({
          name:
            'leaves two UNJUDGED models in the order they arrived, rather than inventing a lead between '
              + 'them: neither has any evidence, so neither can be ahead',
          fn: async () => {
            /**
             Second model with no disinterested ballots at all.
             */
            const alsoUnjudged = standingOf({
              modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              candidates: 7,
              disinterestedBallots: 0,
              disinterestedVotes: 0,
            },);

            expect(rankStandings({
              standings: [
                UNJUDGED,
                alsoUnjudged,
              ],
            },).map(function idOf(standing,): string {
              return standing.modelId;
            },),).toEqual([
              SEAT_SYNTHETIC_VISION_WITHHELD,
              SEAT_SYNTHETIC_TEXT_EVERYWHERE,
            ],);
          },
        },),

        it({
          name:
            'returns a NEW list and leaves the caller`s alone, so a report that ranks the tally it is '
              + 'still accumulating into does not reorder that tally underneath it',
          fn: async () => {
            /**
             Caller's list, in an order the ranking must change.
             */
            const given: readonly ProducerStanding[] = [
              UNJUDGED,
              LEADER,
            ];

            expect(rankStandings({ standings: given, },),).not.toBe(given,);
            expect(given.map(function idOf(standing,): string {
              return standing.modelId;
            },),).toEqual([
              SEAT_SYNTHETIC_VISION_WITHHELD,
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            ],);
          },
        },),

        it({
          name: 'ranks an empty tally to an empty list, rather than raising on a calibration that seated nobody',
          fn: async () => {
            expect(rankStandings({ standings: [], },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
