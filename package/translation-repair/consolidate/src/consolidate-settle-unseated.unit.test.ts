/**
 Tests for the exits of a settlement that cannot seat two judges in its final round (owner, 2026-10-29, "Ship the
 two-ballot text"): the exit that buys no slate, the exit that keeps the standing without one, and the exit after a
 slate round that heard too few judges.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { LaneText, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { TranslateStageResult, } from '@monochromatic-dev/translation-repair-translate-stage';
import {
  type ConsolidationSettlement,
  settleRefusedStanding,
  settleWithoutSlate,
  shipWinnerBeforeSlate,
} from '../dist/final/node/index.mjs';

/**
 Logger the exits write through, whose output is not under test.
 */
const l = tagged({ tag: 'consolidate-settle-unseated-test', },);

/**
 The slice, whose original and archive rendering nothing here reads.
 */
const SUBJECT = {
  sourceText: '猫猫在窗台上打盹。',
  incumbentText: 'The cat dozed and purred for you.',
};

/**
 What the floor made of a slate with nothing valid on it.
 */
const FLOOR = {
  kind: 'incumbent-only',
  refusedModelIds: [],
} as const;

/**
 The translate lane's text as the lane offer hands it over once it passed the rule.
 */
const TRANSLATE_LANE: LaneText = {
  lane: 'translate',
  text: 'The cat naps on the sill.',
};

/**
 The settlement a slate round that kept the refused standing ends in.
 */
const SETTLED = {
  terminal: 'slate-declined-standing',
  text: SUBJECT.incumbentText,
  floor: FLOOR,
  verdicts: [],
  rewrapped: false,
  demoted: false,
  findings: [],
} as unknown as ConsolidationSettlement;

/**
 A slate round that heard the given judges.

 @param heard - judges the round heard

 @returns Round shaped as far as the exit reads it

 @example
 ```ts
 const decided = roundHeard({ heard: 1, });
 ```
 */
function roundHeard({ heard, }: { readonly heard: number; },): TranslateStageResult {
  return { tally: { ballots: heard, }, } as unknown as TranslateStageResult;
}

await describe({
  name: '',
  children: [
    describe({
      name: settleWithoutSlate.name,
      children: [
        it({
          name: 'KEEPS THE ARCHIVE under a standing the rule refused, saying no wording passed, and leaves an '
            + 'eligible standing alone',
          fn: async () => {
            /**
             The settlement over each standing.
             */
            const [refused, eligible,] = [false, true,].map(function settleOver(standingEligible,) {
              return settleWithoutSlate({
                terminal: 'incumbent-only',
                standingEligible,
                standingText: SUBJECT.incumbentText,
                floor: FLOOR,
                verdicts: [],
                producedFindings: [],
                subject: SUBJECT,
                sliceIndex: 3,
                l,
              },);
            },);
            expect({
              refusedKept: refused?.archiveKept,
              refusedReason: refused?.archiveReason,
              eligibleKept: eligible?.archiveKept,
            },).toEqual({
              refusedKept: true,
              refusedReason: 'no-wording-passed',
              eligibleKept: undefined,
            },);
          },
        },),
      ],
    },),
    describe({
      name: shipWinnerBeforeSlate.name,
      children: [
        it({
          name: 'SHIPS THE TRANSLATE TEXT only over a refused standing, with a two-ballot text on the offer and '
            + 'fewer slate judges reachable than a winner needs, and asks the slate in every other case',
          fn: async () => {
            /**
             The arguments every case shares.
             */
            const shared = {
              standingText: SUBJECT.incumbentText,
              floor: FLOOR,
              verdicts: [],
              producedFindings: [],
              subject: SUBJECT,
              sliceIndex: 3,
              l,
            } as const;
            expect({
              ships: shipWinnerBeforeSlate({
                ...shared,
                standingEligible: false,
                laneTexts: [TRANSLATE_LANE,],
                translateWinnerBallots: 2,
                slateReachable: 1,
              },).kind,
              eligible: shipWinnerBeforeSlate({
                ...shared,
                standingEligible: true,
                laneTexts: [TRANSLATE_LANE,],
                translateWinnerBallots: 2,
                slateReachable: 1,
              },).kind,
              oneBallot: shipWinnerBeforeSlate({
                ...shared,
                standingEligible: false,
                laneTexts: [TRANSLATE_LANE,],
                translateWinnerBallots: 1,
                slateReachable: 1,
              },).kind,
              notOffered: shipWinnerBeforeSlate({
                ...shared,
                standingEligible: false,
                laneTexts: [],
                translateWinnerBallots: 2,
                slateReachable: 1,
              },).kind,
              uncounted: shipWinnerBeforeSlate({
                ...shared,
                standingEligible: false,
                laneTexts: [TRANSLATE_LANE,],
                translateWinnerBallots: 2,
              },).kind,
              seated: shipWinnerBeforeSlate({
                ...shared,
                standingEligible: false,
                laneTexts: [TRANSLATE_LANE,],
                translateWinnerBallots: 2,
                slateReachable: 2,
              },).kind,
            },).toEqual({
              ships: 'ships',
              eligible: 'asks',
              oneBallot: 'asks',
              notOffered: 'asks',
              uncounted: 'asks',
              seated: 'asks',
            },);
          },
        },),
      ],
    },),
    describe({
      name: settleRefusedStanding.name,
      children: [
        it({
          name: 'SHIPS THE TRANSLATE TEXT after a slate round that heard fewer than two judges, and keeps the '
            + 'archive with the reason otherwise',
          fn: async () => {
            /**
             The arguments every case shares.
             */
            const shared = {
              settlement: SETTLED,
              laneTexts: [TRANSLATE_LANE,],
              subject: SUBJECT,
              sliceIndex: 3,
              l,
            } as const;
            /**
             The exit after a round that heard one judge, with and without a two-ballot text.
             */
            const unseatedWithText = settleRefusedStanding({
              ...shared,
              decided: roundHeard({ heard: 1, },),
              translateWinnerBallots: 2,
            },);
            const unseatedOneBallot = settleRefusedStanding({
              ...shared,
              decided: roundHeard({ heard: 1, },),
              translateWinnerBallots: 1,
            },);
            const seated = settleRefusedStanding({
              ...shared,
              decided: roundHeard({ heard: 2, },),
              translateWinnerBallots: 2,
            },);
            expect({
              winner: unseatedWithText.translateWinnerShipped,
              winnerText: unseatedWithText.text,
              unseatedReason: unseatedOneBallot.archiveReason,
              seatedReason: seated.archiveReason,
            },).toEqual({
              winner: true,
              winnerText: TRANSLATE_LANE.text,
              unseatedReason: 'final-round-unseated',
              seatedReason: 'slate-declined',
            },);
          },
        },),
      ],
    },),
  ],
},);
