/**
 Tests for the lines the read step prints once the checks of a bench pass: the figures of each arm with the defect-only
 count beside the headline, how the asks of each row ended, and the verdict of the decision rule over the four arms.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ArmLog,
  benchReportLines,
  decideEffort,
} from '../../dist/final/node/index.mjs';
import {
  armLogOf,
  type KindFixed,
  tallyOf,
} from '../editor-fidelity-arm-log.test-fixture.ts';
import {
  HIGH_CARD,
  LOW_CARD,
  MEDIUM_CARD,
} from '../editor-fidelity.test-fixture.ts';

/**
 One arm of a bench.

 @param arm - the arm's name
 @param cardId - the card it seated
 @param insertion - counts of the insertion trials
 @param alteration - counts of the alteration trials

 @returns The arm's log, two rows of which state their stops

 @example
 ```ts
 const log = benchArm({ arm: 'low', cardId: LOW_CARD, insertion, alteration, },);
 ```
 */
function benchArm(
  {
    arm,
    cardId,
    insertion,
    alteration,
  }: {
    readonly arm: string;
    readonly cardId: RosterModelId;
    readonly insertion: KindFixed;
    readonly alteration: KindFixed;
  },
): ArmLog {
  return armLogOf({
    arm,
    tally: tallyOf({
      cardId,
      insertion,
      alteration,
      unusable: 1,
      refused: 2,
    },),
    stops: ['ok', 'refused+ok',],
    seat: `SEAT ${cardId} asked=40 usable=39 unusable=1 threw=0 silent=0 abandoned=1 unreachable=0`,
    benchSpentUsd: 0,
    guardUsd: 0.5,
  },);
}

/**
 Counts of the insertion trials, of which the given number were fixed cleanly.
 */
const insertionOf = (fixed: number, defectOnly = 0,): KindFixed => ({ trials: 22, fixed, defectOnly, });

/**
 Counts of the alteration trials, of which the given number were fixed cleanly.
 */
const alterationOf = (fixed: number, defectOnly = 0,): KindFixed => ({ trials: 18, fixed, defectOnly, });

/**
 A bench in which high fixes twelve more than the mediums' twenty, and low is level with them.
 */
const BENCH: readonly ArmLog[] = [
  benchArm({ arm: 'low', cardId: LOW_CARD, insertion: insertionOf(11, 1,), alteration: alterationOf(9, 2,), },),
  benchArm({ arm: 'medium-a', cardId: MEDIUM_CARD, insertion: insertionOf(11,), alteration: alterationOf(9,), },),
  benchArm({ arm: 'medium-b', cardId: MEDIUM_CARD, insertion: insertionOf(11,), alteration: alterationOf(9,), },),
  benchArm({ arm: 'high', cardId: HIGH_CARD, insertion: insertionOf(18, 3,), alteration: alterationOf(14, 4,), },),
];

await describe({
  name: benchReportLines.name,
  children: [
    it({
      name: 'PRINTS THE FIGURES OF EACH ARM with the clean fix count pooled and for each kind, the defect-only count '
        + 'beside it as a secondary count, and what went unusable, was refused, threw and was abandoned',
      fn: async () => {
        const lines = benchReportLines({ arms: BENCH, },);
        expect(lines[0],).toBe(
          `ARM low (${LOW_CARD}): clean fix 20 of 40 (insertion 11 of 22, alteration 9 of 18); `
            + 'defect only 3 (insertion 1, alteration 2); unusable 1, refused 2, threw 0, abandoned 1',
        );
        expect(lines.filter(function isArm(line,): boolean {
          return line.startsWith('ARM ',);
        },).length,).toBe(4,);
        expect(lines.find(function isHigh(line,): boolean {
          return line.startsWith('ARM high ',);
        },),).toContain('clean fix 32 of 40 (insertion 18 of 22, alteration 14 of 18); defect only 7 (insertion 3, alteration 4)',);
      },
    },),

    it({
      name: 'PRINTS HOW THE ASKS OF EVERY ROW ENDED, one line for each arm, the row numbers beside the stop words',
      fn: async () => {
        const stops = benchReportLines({ arms: BENCH, },).filter(function isStops(line,): boolean {
          return line.startsWith('STOPS ',);
        },);
        expect(stops,).toStrictEqual([
          'STOPS low: row 1 ok; row 2 refused+ok',
          'STOPS medium-a: row 1 ok; row 2 refused+ok',
          'STOPS medium-b: row 1 ok; row 2 refused+ok',
          'STOPS high: row 1 ok; row 2 refused+ok',
        ],);
      },
    },),

    it({
      name: 'THROWS FOR OTHER THAN FOUR ARMS, since a verdict over fewer is not the rule\'s and the read step prints '
        + 'figures only after the bench checks have passed',
      fn: async () => {
        expect(function readsThree(): void {
          benchReportLines({ arms: BENCH.slice(0, 3,), },);
        },).toThrow('four arms',);
      },
    },),

    it({
      name: 'PRINTS THE VERDICT OF THE DECISION RULE LAST, the one decideEffort gives for the counts the logs state, so the '
        + 'read step and the rule cannot differ',
      fn: async () => {
        const lines = benchReportLines({ arms: BENCH, },);
        /**
         The verdict for the same counts, by the rule itself.
         */
        const expected = decideEffort({
          low: { insertion: { trials: 22, fixed: 11, }, alteration: { trials: 18, fixed: 9, }, },
          mediumA: { insertion: { trials: 22, fixed: 11, }, alteration: { trials: 18, fixed: 9, }, },
          mediumB: { insertion: { trials: 22, fixed: 11, }, alteration: { trials: 18, fixed: 9, }, },
          high: { insertion: { trials: 22, fixed: 18, }, alteration: { trials: 18, fixed: 14, }, },
        },);
        expect(expected.verdict,).toBe('high',);
        expect(lines.at(-1,),).toBe(expected.said,);
        expect(lines.at(-1,)?.startsWith('VERDICT high (beats-medium): ',),).toBe(true,);
      },
    },),
  ],
},);
